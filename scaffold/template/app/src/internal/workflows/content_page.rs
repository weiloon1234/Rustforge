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
        .map(|r| r.into_row())
        .ok_or_else(|| AppError::NotFound(t("Page not found")))
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminContentPageUpdateInput,
) -> Result<ContentPageView, AppError> {
    let tag = normalize_tag(&req.tag)?;
    let existing = detail(state, id).await?;

    if matches!(existing.is_system, ContentPageSystemFlag::Yes) {
        if existing.tag != tag {
            return Err(AppError::Forbidden(t("Cannot change tag for system page")));
        }

        let existing_title = existing.title_translations.unwrap_or_default();
        let title_changed = existing_title.en != req.title.en.clone().unwrap_or_default()
            || existing_title.zh != req.title.zh.clone().unwrap_or_default();
        if title_changed {
            return Err(AppError::Forbidden(t(
                "Cannot change title for system page",
            )));
        }
    }

    let affected = ContentPage::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_tag(tag)
        .set_title_input(Some(req.title))
        .set_content_input(Some(req.content))
        .set_cover_input(req.cover)
        .save()
        .await
        .map_err(AppError::from)?;

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
