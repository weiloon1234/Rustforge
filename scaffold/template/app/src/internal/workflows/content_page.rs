use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{ContentPageModel, ContentPageRecord, ContentPageSystemFlag};

use crate::{
    contracts::api::v1::admin::content_page::AdminContentPageUpdateInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<ContentPageRecord, AppError> {
    ContentPageModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Page not found")))
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminContentPageUpdateInput,
) -> Result<ContentPageRecord, AppError> {
    let tag = normalize_tag(&req.tag)?;
    let existing = detail(state, id).await?;

    if matches!(existing.is_system, ContentPageSystemFlag::Yes) {
        if existing.tag != tag {
            return Err(AppError::Forbidden(t("Cannot change tag for system page")));
        }

        let existing_title = existing.title_translations.clone().unwrap_or_default();
        let title_changed = existing_title.en != req.title.en.clone().unwrap_or_default()
            || existing_title.zh != req.title.zh.clone().unwrap_or_default();
        if title_changed {
            return Err(AppError::Forbidden(t(
                "Cannot change title for system page",
            )));
        }
    }

    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let affected = ContentPageModel::query(conn.clone())
        .where_col(generated::models::ContentPageCol::ID, Op::Eq, id)
        .patch()
        .assign(generated::models::ContentPageCol::TAG, tag)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Page not found")));
    }

    existing
        .upsert_title(conn.clone(), Some(req.title))
        .await
        .map_err(AppError::from)?;
    existing
        .upsert_content(conn.clone(), Some(req.content))
        .await
        .map_err(AppError::from)?;
    existing
        .upsert_cover(conn, req.cover)
        .await
        .map_err(AppError::from)?;

    scope.commit().await.map_err(AppError::from)?;

    detail(state, id).await
}

pub async fn remove(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let existing = detail(state, id).await?;
    if matches!(existing.is_system, ContentPageSystemFlag::Yes) {
        return Err(AppError::Forbidden(t("Cannot delete system page")));
    }

    let affected = ContentPageModel::query(DbConn::pool(&state.db))
        .where_col(generated::models::ContentPageCol::ID, Op::Eq, id)
        .delete()
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
