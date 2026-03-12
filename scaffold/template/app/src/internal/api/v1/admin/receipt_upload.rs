use axum::{body::Bytes, extract::Multipart};
use core_i18n::t;
use core_web::error::AppError;

pub const MAX_RECEIPT_BYTES: usize = 5 * 1024 * 1024;

pub async fn parse_receipt_multipart(
    mut multipart: Multipart,
) -> Result<(Option<String>, String, Bytes), AppError> {
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let field_name = field
            .name()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest(t("Invalid multipart field")))?;

        match field_name.as_str() {
            "file" | "fileUpload" | "receipt" => {
                if bytes.is_some() {
                    return Err(AppError::BadRequest(t("Invalid multipart field")));
                }
                if field.file_name().is_none() {
                    return Err(AppError::BadRequest(t("Invalid upload file field")));
                }
                file_name = field.file_name().map(ToString::to_string);
                content_type = Some(
                    field
                        .content_type()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                );
                let data = field.bytes().await.map_err(AppError::from)?;
                if data.is_empty() {
                    return Err(AppError::BadRequest(t("Uploaded file is empty")));
                }
                if data.len() > MAX_RECEIPT_BYTES {
                    return Err(AppError::BadRequest(t("File exceeds maximum allowed size")));
                }
                bytes = Some(data);
            }
            _ => {}
        }
    }

    let bytes = bytes.ok_or_else(|| AppError::BadRequest(t("Missing required field: file")))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    Ok((file_name, content_type, bytes))
}

pub fn validate_attachment_allowed(
    allowed: &[String],
    filename: Option<&str>,
    content_type: &str,
) -> Result<(), AppError> {
    if allowed.is_empty() {
        return Ok(());
    }
    let ct = content_type.to_ascii_lowercase();
    for rule in allowed {
        let rule = rule.to_ascii_lowercase();
        if rule == "*" || rule == "*/*" || ct == rule {
            return Ok(());
        }
        if rule.starts_with('.') {
            if let Some(name) = filename.map(|n| n.to_ascii_lowercase()) {
                if name.ends_with(&rule) {
                    return Ok(());
                }
            }
        }
    }
    Err(AppError::BadRequest(t("Attachment type not allowed")))
}

pub fn build_attachment_url(path: &str, base: Option<&str>) -> String {
    let raw = path.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("//")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("data:")
        || raw.starts_with("blob:")
    {
        return raw.to_string();
    }
    let Some(base) = base.map(str::trim).filter(|v| !v.is_empty()) else {
        return raw.to_string();
    };
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        raw.trim_start_matches('/')
    )
}
