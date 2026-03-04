use std::collections::{BTreeMap, HashMap, HashSet};

use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{ContentPage, ContentPageSystemFlag, ContentPageView};

use crate::{
    contracts::api::v1::admin::content_page::AdminContentPageUpdateInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<ContentPageView, AppError> {
    ContentPage::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Page not found")))
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminContentPageUpdateInput,
) -> Result<ContentPageView, AppError> {
    let tag = normalize_tag(&req.tag)?;
    let title = normalize_localized_map(&req.title.to_map());
    let content = normalize_localized_html_map(&req.content.to_map());
    let cover = normalize_localized_map(&req.cover.to_map());
    let has_cover_update = cover.values().any(|value| !value.is_empty());

    let existing = detail(state, id).await?;
    if matches!(existing.is_system, ContentPageSystemFlag::Yes) {
        if existing.tag != tag {
            return Err(AppError::Forbidden(t("Cannot change tag for system page")));
        }

        let existing_title = localized_text_to_map(existing.title_translations.as_ref());
        if existing_title != title {
            return Err(AppError::Forbidden(t(
                "Cannot change title for system page",
            )));
        }
    }

    let title_langs = localized_text_from_map(&title, true, "title")?;
    let content_langs = localized_text_from_map(&content, true, "content")?;

    let mut update = ContentPage::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_tag(tag)
        .set_title_langs(title_langs)
        .set_content_langs(content_langs);

    if has_cover_update {
        let cover_langs = localized_text_from_map(&cover, false, "cover")?;
        update = update.set_cover_langs(cover_langs);
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Page not found")));
    }

    detail(state, id).await
}

pub async fn remove(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let existing = detail(state, id).await?;
    if matches!(existing.is_system, ContentPageSystemFlag::Yes) {
        return Err(AppError::Forbidden(t("Cannot delete system page")));
    }

    let affected = ContentPage::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Page not found")));
    }

    Ok(())
}

fn normalize_tag(input: &str) -> Result<String, AppError> {
    let normalized = input.trim().to_ascii_lowercase();
    if normalized.is_empty() || !is_valid_snake_case_tag(&normalized) {
        return Err(AppError::BadRequest(t("Tag must be lowercase snake_case")));
    }
    Ok(normalized)
}

fn is_valid_snake_case_tag(input: &str) -> bool {
    if input.starts_with('_') || input.ends_with('_') {
        return false;
    }
    let mut previous_underscore = false;
    for ch in input.chars() {
        if ch == '_' {
            if previous_underscore {
                return false;
            }
            previous_underscore = true;
            continue;
        }
        previous_underscore = false;
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            return false;
        }
    }
    true
}

fn normalize_localized_map(input: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    input
        .iter()
        .map(|(locale, value)| (locale.to_string(), value.trim().to_string()))
        .collect()
}

fn normalize_localized_html_map(input: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    input
        .iter()
        .map(|(locale, value)| {
            (
                locale.to_string(),
                sanitize_rich_html(value.trim()).trim().to_string(),
            )
        })
        .collect()
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

fn localized_text_to_map(
    localized_text: Option<&generated::LocalizedText>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for &locale in generated::SUPPORTED_LOCALES {
        let value = localized_text
            .map(|current| current.get(locale).to_string())
            .unwrap_or_default();
        out.insert(locale.to_string(), value);
    }
    out
}

fn localized_text_from_map(
    input: &BTreeMap<String, String>,
    require_all_locales: bool,
    field_label: &str,
) -> Result<generated::LocalizedText, AppError> {
    let mut payload = BTreeMap::new();
    for &locale in generated::SUPPORTED_LOCALES {
        let value = input
            .get(locale)
            .map(|item| item.trim().to_string())
            .unwrap_or_default();
        if require_all_locales && value.is_empty() {
            return Err(AppError::BadRequest(format!(
                "{}: {} ({})",
                t("Missing localized value"),
                field_label,
                locale
            )));
        }
        payload.insert(locale.to_string(), value);
    }

    Ok(generated::LocalizedText::from_map(&payload))
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
