use std::collections::BTreeMap;

use axum::extract::Multipart;
use core_i18n::t;
use core_web::error::AppError;

use crate::{contracts::api::v1::admin::page::AdminPageUpdateInput, internal::api::state::AppApiState};

pub async fn parse_page_update_multipart(
    state: &AppApiState,
    mut multipart: Multipart,
) -> Result<AdminPageUpdateInput, AppError> {
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

                let attachment = core_db::platform::attachments::service::process_and_upload_with_type(
                    state.storage.as_ref(),
                    "image",
                    "page",
                    &format!("cover_{locale}"),
                    filename.as_deref(),
                    &content_type,
                    bytes,
                )
                .await
                .map_err(AppError::from)?;

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

    Ok(AdminPageUpdateInput {
        tag,
        title,
        content,
        cover,
    })
}

fn localized_field_locale<'a>(field_name: &'a str, prefix: &str) -> Result<Option<&'a str>, AppError> {
    let Some(locale_raw) = field_name.strip_prefix(prefix).and_then(|rest| rest.strip_prefix('.'))
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
