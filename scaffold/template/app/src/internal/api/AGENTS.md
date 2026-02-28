# API Handlers

Route handlers in `api/v1/`. Handlers are **thin** — parse input, call workflow, wrap in response.

## Handler Pattern

```rust
use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::{AsyncContractJson, ContractJson},
    error::AppError,
    openapi::{
        with_permission_check_get_with, with_permission_check_post_with,
        with_permission_check_patch_with, with_permission_check_delete_with,
        ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};
use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create, AdminGuard, PermissionMode::Any,
                [Permission::ArticleManage.as_str()],
                |op| op.summary("Create article").tag("Articles"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail, AdminGuard, PermissionMode::Any,
                [Permission::ArticleRead.as_str()],
                |op| op.summary("Get article").tag("Articles"),
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: AsyncContractJson<CreateArticleInput>,
) -> Result<ApiResponse<ArticleOutput>, AppError> {
    let article = workflow::create(&state, &auth, req.0).await?;
    Ok(ApiResponse::success(ArticleOutput::from(article), &t("Article created")))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<ArticleOutput>, AppError> {
    let article = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(ArticleOutput::from(article), &t("Article loaded")))
}
```

## Extractors

| Extractor | When to use |
|-----------|-------------|
| `ContractJson<T>` | Sync validation only |
| `AsyncContractJson<T>` | Has `async_unique`/`async_exists` rules |

For update with async validation, validate manually:
```rust
async fn update(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
    req: ContractJson<UpdateInput>,
) -> Result<ApiResponse<Output>, AppError> {
    let req = req.0.with_target_id(id);
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    // ...
}
```

## Router Wiring

Register new domain routers in `api/v1/mod.rs`:
```rust
mod article;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/articles", article::router(state.clone()))
        // ...
}
```

Guarded routes use middleware layer:
```rust
.layer(from_fn_with_state(state, crate::internal::middleware::auth::require_admin))
```

## Auth in Handlers

```rust
// Extract user
auth: AuthUser<AdminGuard>

// Permission check
use core_web::authz::{PermissionMode, ensure_permissions};
ensure_permissions(&auth, PermissionMode::Any, &["article.read"])?;

// Direct check
auth.has_permission("article.manage")
```

## State

`AppApiState` in `state.rs` holds `db`, `auth`, `storage`, `mailer`, registries. Extend it when adding new shared resources.
