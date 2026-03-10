use axum::extract::State;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    extract::CleanJson,
    openapi::{with_permission_check_post_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::introducer_change::{
        ChangeIntroducerInput, ChangeIntroducerOutput,
    },
    internal::{
        api::state::AppApiState,
        workflows::introducer_change as workflow,
    },
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                change_introducer,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserChangeIntroducer.as_str()],
                |op| {
                    op.summary("Change user introducer")
                        .tag("Admin Introducer Changes")
                },
            ),
        )
        .with_state(state)
}

async fn change_introducer(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    CleanJson(req): CleanJson<ChangeIntroducerInput>,
) -> Result<ApiResponse<ChangeIntroducerOutput>, AppError> {
    let log = workflow::change_introducer(
        &state,
        auth.user.id,
        &req.user_username,
        &req.new_introducer_username,
        req.remark,
    )
    .await?;

    Ok(ApiResponse::success(
        ChangeIntroducerOutput {
            id: log.id.into(),
            user_id: log.user_id.into(),
            from_user_id: log.from_user_id.map(|id| id.into()),
            to_user_id: log.to_user_id.into(),
            admin_id: log.admin_id.into(),
            remark: log.remark,
            created_at: log.created_at,
        },
        &t("Introducer changed"),
    ))
}
