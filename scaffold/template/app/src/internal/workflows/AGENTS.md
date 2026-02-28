# Workflows

Business logic functions. One file per domain. Handlers call these — keep DB queries, permission checks, and orchestration here.

## Pattern

```rust
use core_db::common::sql::{DbConn, Op, generate_snowflake_i64};
use core_i18n::t;
use core_web::{auth::AuthUser, error::AppError};
use generated::{guards::AdminGuard, models::{Article, ArticleView, ArticleQuery}};
use crate::internal::api::state::AppApiState;

pub async fn detail(state: &AppApiState, id: i64) -> Result<ArticleView, AppError> {
    Article::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))
}

pub async fn create(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    req: CreateArticleInput,
) -> Result<ArticleView, AppError> {
    Article::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_title(req.title.trim().to_string())
        .set_slug(req.slug.trim().to_ascii_lowercase())
        .save()
        .await
        .map_err(AppError::from)
}

pub async fn update(state: &AppApiState, id: i64, req: UpdateArticleInput) -> Result<ArticleView, AppError> {
    let mut update = Article::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id);

    if let Some(title) = req.title {
        update = update.set_title(title.trim().to_string());
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Article not found")));
    }

    detail(state, id).await
}

pub async fn remove(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = Article::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Article not found")));
    }
    Ok(())
}
```

## Generated Model API

| Operation | Code |
|-----------|------|
| Create handle | `Model::new(DbConn::pool(&db), None)` |
| Insert | `.insert().set_field(val).save()` → `ModelView` |
| Update | `.update().where_id(Op::Eq, id).set_field(val).save()` → affected rows |
| Delete | `.delete(id)` → affected rows (soft-delete if enabled) |
| Find by PK | `.find(id)` → `Option<ModelView>` |
| Query | `ModelQuery::new(...).where_field(Op::Eq, val).first()` → `Option<ModelView>` |
| Hashed field | `.set_password(&plain_text).map_err(AppError::from)?` (returns Result) |

IDs use snowflake: `generate_snowflake_i64()`.
