use std::collections::{BTreeMap, HashMap, HashSet};

use axum::extract::Multipart;
use core_i18n::t;
use core_web::error::AppError;
use generated::localized::LocalizedInput;
use uuid::Uuid;
use validator::Validate;

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

    // Sanitize HTML content before constructing input
    let content: BTreeMap<String, String> = content
        .into_iter()
        .map(|(locale, value)| (locale, sanitize_rich_html(&value).trim().to_string()))
        .collect();

    let title = LocalizedInput::from_map(&title);
    let content = LocalizedInput::from_map(&content);
    let cover = if cover.is_empty() {
        None
    } else {
        Some(LocalizedInput::from_map(&cover))
    };

    let input = AdminContentPageUpdateInput {
        tag,
        title,
        content,
        cover,
    };
    if let Err(e) = input.validate() {
        let errors = core_web::extract::validation::transform_validation_errors(e);
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors,
        });
    }
    Ok(input)
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

fn sanitize_rich_html(input: &str) -> String {
    let mut tag_attributes: HashMap<&str, HashSet<&str>> = HashMap::new();
    tag_attributes.insert("a", HashSet::from(["href", "target"]));
    tag_attributes.insert(
        "img",
        HashSet::from(["src", "alt", "title", "width", "height"]),
    );
    tag_attributes.insert("th", HashSet::from(["colspan", "rowspan"]));
    tag_attributes.insert("td", HashSet::from(["colspan", "rowspan"]));
    tag_attributes.insert("ul", HashSet::from(["data-type"]));
    tag_attributes.insert("li", HashSet::from(["data-type", "data-checked"]));
    tag_attributes.insert("input", HashSet::from(["type", "checked", "disabled"]));

    let mut builder = ammonia::Builder::default();
    builder
        .tags(HashSet::from([
            "p",
            "br",
            "ul",
            "ol",
            "li",
            "h2",
            "h3",
            "strong",
            "em",
            "s",
            "blockquote",
            "code",
            "pre",
            "hr",
            "a",
            "u",
            "mark",
            "img",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "label",
            "input",
            "span",
            "div",
        ]))
        .tag_attributes(tag_attributes)
        .url_schemes(HashSet::from(["http", "https"]))
        .url_relative(ammonia::UrlRelative::PassThrough)
        .link_rel(Some("noopener noreferrer nofollow"))
        .clean_content_tags(HashSet::from(["script", "style", "iframe"]));

    builder.clean(input).to_string()
}

#[cfg(test)]
mod tests {
    use super::sanitize_rich_html;

    #[test]
    fn sanitize_rich_html_keeps_links_without_panicking() {
        let output =
            sanitize_rich_html(r#"<p><a href="https://example.com" target="_blank">x</a></p>"#);
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains("rel=\"noopener noreferrer nofollow\""));
    }

    #[test]
    fn sanitize_rich_html_removes_script() {
        let output = sanitize_rich_html("<p>ok</p><script>alert('x')</script>");
        assert!(output.contains("<p>ok</p>"));
        assert!(!output.contains("script"));
    }
}
