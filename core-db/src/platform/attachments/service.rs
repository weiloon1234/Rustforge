#![allow(dead_code)]
use bytes::Bytes;
use mime_guess::MimeGuess;
use image::{GenericImageView, ImageEncoder};
use uuid::Uuid;

use crate::generated;
use crate::infra::storage::Storage;
use crate::platform::attachments::types::{AttachmentRules, AttachmentUploadDto};

/// Build a storage key like `owner/field/YYYY/MM/DD/uuid.ext`.
/// `ext` may include or omit the leading dot.
pub fn build_object_key(owner_type: &str, field: &str, ext: &str) -> String {
    let ext = ext.trim();
    let ext = if ext.is_empty() {
        String::new()
    } else if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{}", ext)
    };
    let now = time::OffsetDateTime::now_utc();
    let uuid = Uuid::new_v4();
    format!(
        "{}/{:04}/{:02}/{:02}/{}/{}{}",
        owner_type.trim_matches('/'),
        now.year(),
        now.month() as u8,
        now.day(),
        field.trim_matches('/'),
        uuid,
        ext
    )
}

/// Validate mime/ext against rules. `filename` is optional; falls back to content-type sniff.
pub fn validate_attachment(rules: &AttachmentRules, filename: Option<&str>, content_type: &str) -> anyhow::Result<()> {
    if rules.allowed.is_empty() {
        return Ok(());
    }
    let mut ok = false;

    let guess = filename
        .and_then(|name| MimeGuess::from_path(name).first_raw())
        .map(|m: &str| m.to_string());

    let ct = content_type.to_ascii_lowercase();
    for rule in &rules.allowed {
        let rule = rule.to_ascii_lowercase();
        if rule == "*/*" || rule == "*" {
            ok = true;
            break;
        }
        if rule.starts_with("image/") && ct.starts_with("image/") {
            ok = true;
            break;
        }
        if rule.starts_with('.') {
            if let Some(name) = filename {
                if name.to_ascii_lowercase().ends_with(&rule) {
                    ok = true;
                    break;
                }
            }
        } else if ct == rule {
            ok = true;
            break;
        } else if let Some(g) = &guess {
            if g == &rule {
                ok = true;
                break;
            }
        }
    }

    if !ok {
        anyhow::bail!("Attachment type not allowed (allowed: {:?}, got: {} {:?})", rules.allowed, content_type, filename);
    }
    Ok(())
}

/// Resize image bytes if a resize rule is present. For now, just passes through (stub).
pub fn maybe_resize(rules: &AttachmentRules, data: Bytes, content_type: &str) -> anyhow::Result<(Bytes, Option<(i32, i32)>, String)> {
    if !content_type.to_ascii_lowercase().starts_with("image/") {
        return Ok((data, None, content_type.to_string()));
    }
    let Some(resize) = &rules.resize else {
        return Ok((data, None, content_type.to_string()));
    };
    let img = image::load_from_memory(&data)?;
    let (orig_w, orig_h) = img.dimensions();
    let (target_w, target_h) = match (resize.width, resize.height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let h = ((orig_h as f32) * (w as f32 / orig_w as f32)).round().max(1.0) as u32;
            (w, h)
        }
        (None, Some(h)) => {
            let w = ((orig_w as f32) * (h as f32 / orig_h as f32)).round().max(1.0) as u32;
            (w, h)
        }
        (None, None) => return Ok((data, Some((orig_w as i32, orig_h as i32)), content_type.to_string())),
    };
    let resized = img.resize(target_w, target_h, image::imageops::Lanczos3);
    let mut buf = Vec::new();
    let ct_lower = content_type.to_ascii_lowercase();
    if ct_lower.contains("jpeg") || ct_lower.contains("jpg") {
        let quality = resize.quality.unwrap_or(85);
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
        enc.encode_image(&resized)?;
        Ok((Bytes::from(buf), Some((resized.width() as i32, resized.height() as i32)), "image/jpeg".to_string()))
    } else {
        let png_bytes = resized.to_rgba8();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        enc.write_image(
            &png_bytes,
            resized.width(),
            resized.height(),
            image::ColorType::Rgba8.into(),
        )?;
        Ok((Bytes::from(buf), Some((resized.width() as i32, resized.height() as i32)), "image/png".to_string()))
    }
}

/// Convenience to build AttachmentUploadDto after upload.
pub fn make_attachment_input(
    path: String,
    content_type: String,
    size: i64,
    width: Option<i32>,
    height: Option<i32>,
) -> AttachmentUploadDto {
    AttachmentUploadDto::new(path, content_type, size, width, height)
}

/// Validate, optionally resize, upload to storage, and return an AttachmentUploadDto ready for DB.
pub async fn process_and_upload(
    storage: &dyn Storage,
    rules: &AttachmentRules,
    owner_type: &str,
    field: &str,
    filename: Option<&str>,
    content_type: &str,
    data: Bytes,
) -> anyhow::Result<AttachmentUploadDto> {
    validate_attachment(rules, filename, content_type)?;
    let ext = filename
        .and_then(|n| std::path::Path::new(n).extension().and_then(|e| e.to_str()))
        .unwrap_or_else(|| {
            content_type
                .split('/')
                .nth(1)
                .unwrap_or("bin")
        });
    let key = build_object_key(owner_type, field, ext);
    let (data, dims, ct_final) = maybe_resize(rules, data, content_type)?;
    storage.put(&key, data.clone(), &ct_final).await?;
    let size = data.len() as i64;
    let (width, height) = dims.map(|(w, h)| (Some(w), Some(h))).unwrap_or((None, None));
    Ok(make_attachment_input(key, ct_final, size, width, height))
}

/// Fetch rules by type (from generated configs), then process + upload.
pub async fn process_and_upload_with_type(
    storage: &dyn Storage,
    type_name: &str,
    owner_type: &str,
    field: &str,
    filename: Option<&str>,
    content_type: &str,
    data: Bytes,
) -> anyhow::Result<AttachmentUploadDto> {
    let rules = generated::get_attachment_rules(type_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown attachment type '{}'", type_name))?;
    process_and_upload(storage, &rules, owner_type, field, filename, content_type, data).await
}
