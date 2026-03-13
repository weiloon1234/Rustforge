use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use core_db::common::model_observer::scope_observer;
use core_web::error::AppError;
use generated::guards::AdminGuard;
use generated::guards::UserGuard;

use crate::internal::api::state::AppApiState;
use crate::internal::observers::model::AppModelObserver;

pub async fn require_admin(
    state: State<AppApiState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    core_web::auth::require_auth::<AdminGuard, AppApiState>(state, request, next).await
}

/// Admin auth middleware with audit observer scope.
/// Wraps the handler so all model mutations within the request
/// automatically trigger audit logging via lifecycle hooks.
pub async fn require_admin_with_audit(
    state: State<AppApiState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let db = state.db.clone();

    // First authenticate
    let token = core_web::auth::extract_bearer_token(request.headers())
        .ok_or_else(|| AppError::Unauthorized("Missing access token".to_string()))?;

    let auth_user =
        core_web::auth::authenticate_token::<AdminGuard>(core_web::auth::AuthState::auth_db(&*state), &token)
            .await?;

    let admin_id = auth_user.user.id;
    request.extensions_mut().insert(auth_user);

    // Run the rest of the request with the audit observer in scope
    let observer = Arc::new(AppModelObserver::new(db, admin_id));

    let response = scope_observer(observer, || next.run(request)).await;
    Ok(response)
}

pub async fn require_user(
    state: State<AppApiState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    core_web::auth::require_auth::<UserGuard, AppApiState>(state, request, next).await
}
