use std::collections::BTreeMap;

use axum::extract::Multipart;
use core_i18n::t;
use core_web::error::AppError;
use uuid::Uuid;

use crate::{
    contracts::api::v1::admin::content_page::AdminContentPageUpdateInput,
    internal::api::state::AppApiState,
};

pub async fn parse_content_page_update_multipart(
    state: &AppApiState,
    mut multipart: Multipart,
) -> Result<AdminContentPageUpdateInput, AppError> {
    let mut tag: Option<String> = None;
    let mut title: BTreeMap<String, String> = BTreeMap::new();
    let mut content: BTreeMap<String, String> = BTreeMap::new();
    let mut cover: BTreeMap<String, String> = BTreeMap::new();

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let field_name = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest(t("Invalid multipart field")))?;

        if field_name == "tag" {
            let value = field.text().await.map_err(AppError::from)?;
            tag = Some(value.trim().to_string());
            continue;
        }

        if let Some(locale) = localized_field_locale(&field_name, "title")? {
            let value = field.text().await.map_err(AppError::from)?;
            title.insert(locale.to_string(), value.trim().to_string());
            continue;
        }

        if let Some(locale) = localized_field_locale(&field_name, "content")? {
            let value = field.text().await.map_err(AppError::from)?;
            content.insert(locale.to_string(), value.trim().to_string());
            continue;
        }

        if let Some(locale) = localized_field_locale(&field_name, "cover")? {
            if field.file_name().is_some() {
                let filename = field.file_name().map(ToString::to_string);
                let content_type = field
                    .content_type()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let bytes = field.bytes().await.map_err(AppError::from)?;
                if bytes.is_empty() {
                    continue;
                }

                let attachment = upload_content_page_cover(
                    state,
                    locale,
                    filename.as_deref(),
                    &content_type,
                    bytes,
                )
                .await?;

                cover.insert(locale.to_string(), attachment.path);
            } else {
                let value = field.text().await.map_err(AppError::from)?;
                let value = value.trim();
                if !value.is_empty() {
                    cover.insert(locale.to_string(), value.to_string());
                }
            }
            continue;
        }

        return Err(AppError::BadRequest(t("Unknown multipart field")));
    }

    let tag = tag
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest(t("Missing required field: tag")))?;

    ensure_required_locales("title", &title)?;
    ensure_required_locales("content", &content)?;

    Ok(AdminContentPageUpdateInput {
        tag,
        title: generated::LocalizedText::from_map(&title),
        content: generated::LocalizedText::from_map(&content),
        cover: generated::LocalizedText::from_map(&cover),
    })
}

async fn upload_content_page_cover(
    state: &AppApiState,
    locale: &str,
    filename: Option<&str>,
    content_type: &str,
    bytes: axum::body::Bytes,
) -> Result<core_db::platform::attachments::AttachmentUploadDto, AppError> {
    let rules = generated::get_attachment_rules("image")
        .ok_or_else(|| AppError::BadRequest(t("Unknown attachment type: image")))?;

    validate_attachment_allowed(&rules.allowed, filename, content_type)?;

    let ext = filename
        .and_then(|name| {
            std::path::Path::new(name)
                .extension()
                .and_then(|item| item.to_str())
        })
        .unwrap_or_else(|| content_type.split('/').nth(1).unwrap_or("bin"));

    let now = time::OffsetDateTime::now_utc();
    let object_key = format!(
        "content_page/{:04}/{:02}/{:02}/cover_{locale}/{}.{}",
        now.year(),
        now.month() as u8,
        now.day(),
        Uuid::new_v4(),
        ext
    );

    state
        .storage
        .put(&object_key, bytes.clone(), content_type)
        .await
        .map_err(AppError::from)?;

    let mut dto = core_db::platform::attachments::AttachmentUploadDto::new(
        object_key,
        content_type.to_string(),
        bytes.len() as i64,
        None,
        None,
    );
    if let Some(name) = filename {
        dto = dto.with_name(name.to_string());
    }
    Ok(dto)
}

fn validate_attachment_allowed(
    allowed: &[String],
    filename: Option<&str>,
    content_type: &str,
) -> Result<(), AppError> {
    if allowed.is_empty() {
        return Ok(());
    }

    let ct = content_type.to_ascii_lowercase();
    let filename_lower = filename.map(|item| item.to_ascii_lowercase());

    let mut ok = false;
    for rule in allowed {
        let rule = rule.to_ascii_lowercase();
        if rule == "*" || rule == "*/*" {
            ok = true;
            break;
        }
        if rule.starts_with("image/") && ct.starts_with("image/") {
            ok = true;
            break;
        }
        if rule.starts_with('.') {
            if let Some(name) = filename_lower.as_deref() {
                if name.ends_with(&rule) {
                    ok = true;
                    break;
                }
            }
            continue;
        }
        if ct == rule {
            ok = true;
            break;
        }
    }

    if ok {
        Ok(())
    } else {
        Err(AppError::BadRequest(t("Attachment type not allowed")))
    }
}

fn localized_field_locale<'a>(
    field_name: &'a str,
    prefix: &str,
) -> Result<Option<&'a str>, AppError> {
    let Some(locale_raw) = field_name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('.'))
    else {
        return Ok(None);
    };

    let locale = normalize_locale(locale_raw)?;
    Ok(Some(locale))
}

fn normalize_locale(raw: &str) -> Result<&'static str, AppError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::BadRequest(t("Invalid locale field key")));
    }
    core_i18n::match_supported_locale(raw)
        .ok_or_else(|| AppError::BadRequest(t("Unsupported locale")))
}

fn ensure_required_locales(
    field_label: &str,
    values: &BTreeMap<String, String>,
) -> Result<(), AppError> {
    for &locale in generated::SUPPORTED_LOCALES {
        let has_value = values
            .get(locale)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        if !has_value {
            return Err(AppError::BadRequest(format!(
                "{}: {} ({})",
                t("Missing localized value"),
                field_label,
                locale
            )));
        }
    }
    Ok(())
}
