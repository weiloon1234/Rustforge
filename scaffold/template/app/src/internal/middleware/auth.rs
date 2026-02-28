use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use core_web::error::AppError;
use generated::guards::AdminGuard;

use crate::internal::api::state::AppApiState;

pub async fn require_admin(
    state: State<AppApiState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    core_web::auth::require_auth::<AdminGuard, AppApiState>(state, request, next).await
}
