use axum::extract::{Multipart, Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_patch_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::content_page::{
        AdminContentPageDeleteOutput, AdminContentPageOutput, AdminContentPageUpdateOutput,
    },
    internal::{api::state::AppApiState, workflows::content_page as workflow},
};

use super::content_page_multipart;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::ContentPageRead.as_str(),
                    Permission::ContentPageManage.as_str(),
                ],
                |op| op.summary("Get page detail").tag("Admin Page"),
            )
            .merge(with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ContentPageManage.as_str()],
                |op| op.summary("Update page").tag("Admin Page"),
            ))
            .merge(with_permission_check_delete_with(
                remove,
                AdminGuard,
                PermissionMode::Any,
                [Permission::ContentPageManage.as_str()],
                |op| op.summary("Delete page").tag("Admin Page"),
            )),
        )
        .with_state(state)
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminContentPageOutput>, AppError> {
    let page = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(
        AdminContentPageOutput::from(page),
        &t("Page loaded"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<AdminContentPageUpdateOutput>, AppError> {
    let req =
        content_page_multipart::parse_content_page_update_multipart(&state, multipart).await?;
    let page = workflow::update(&state, id, req).await?;
    Ok(ApiResponse::success(
        AdminContentPageUpdateOutput::from(page),
        &t("Page updated"),
    ))
}

async fn remove(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminContentPageDeleteOutput>, AppError> {
    workflow::remove(&state, id).await?;
    Ok(ApiResponse::success(
        AdminContentPageDeleteOutput { deleted: true },
        &t("Page deleted"),
    ))
}
