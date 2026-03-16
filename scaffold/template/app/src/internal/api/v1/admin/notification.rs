use axum::extract::State;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    error::AppError,
    openapi::{aide::axum::routing::get_with, ApiRouter},
    response::ApiResponse,
};
use generated::guards::AdminGuard;

use crate::{
    contracts::api::v1::admin::notification::AdminNotificationCountsOutput,
    internal::{api::state::AppApiState, workflows::notification},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/counts",
            get_with(counts, |op| {
                op.summary("Get admin notification counts")
                    .tag("Admin Notifications")
            }),
        )
        .with_state(state)
}

async fn counts(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
) -> Result<ApiResponse<AdminNotificationCountsOutput>, AppError> {
    let pending = notification::get_pending_counts(&state.db)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::success(
        AdminNotificationCountsOutput {
            deposit: pending.deposit,
            withdrawal: pending.withdrawal,
        },
        &t("Notification counts"),
    ))
}
