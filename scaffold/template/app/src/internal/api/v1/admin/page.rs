use axum::{
    extract::{Multipart, Path, State},
};
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
    contracts::api::v1::admin::page::{
        AdminPageDeleteOutput, AdminPageOutput, AdminPageUpdateOutput,
    },
    internal::{
        api::state::AppApiState,
        workflows::page as workflow,
    },
};

use super::page_multipart;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [Permission::PageRead.as_str(), Permission::PageManage.as_str()],
                |op| op.summary("Get page detail").tag("Admin Page"),
            )
            .merge(with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::PageManage.as_str()],
                |op| op.summary("Update page").tag("Admin Page"),
            ))
            .merge(with_permission_check_delete_with(
                remove,
                AdminGuard,
                PermissionMode::Any,
                [Permission::PageManage.as_str()],
                |op| op.summary("Delete page").tag("Admin Page"),
            )),
        )
        .with_state(state)
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminPageOutput>, AppError> {
    let page = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(AdminPageOutput::from(page), &t("Page loaded")))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    multipart: Multipart,
) -> Result<ApiResponse<AdminPageUpdateOutput>, AppError> {
    let req = page_multipart::parse_page_update_multipart(&state, multipart).await?;
    let page = workflow::update(&state, id, req).await?;
    Ok(ApiResponse::success(
        AdminPageUpdateOutput::from(page),
        &t("Page updated"),
    ))
}

async fn remove(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminPageDeleteOutput>, AppError> {
    workflow::remove(&state, id).await?;
    Ok(ApiResponse::success(
        AdminPageDeleteOutput { deleted: true },
        &t("Page deleted"),
    ))
}
