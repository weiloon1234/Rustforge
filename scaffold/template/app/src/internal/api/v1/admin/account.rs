use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::AsyncContractJson,
    error::AppError,
    extract::CleanJson,
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_patch_with, with_permission_check_post_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::account::{
        AdminBatchResolveEntry, AdminBatchResolveInput, AdminBatchResolveOutput,
        AdminDeleteOutput, AdminOutput, CreateAdminInput, UpdateAdminInput,
    },
    internal::{api::state::AppApiState, workflows::admin as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Create admin").tag("Admin Account"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::AdminRead.as_str(),
                    Permission::AdminManage.as_str(),
                ],
                |op| op.summary("Get admin detail").tag("Admin Account"),
            )
            .merge(with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Update admin").tag("Admin Account"),
            ))
            .merge(with_permission_check_delete_with(
                remove,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Delete admin").tag("Admin Account"),
            )),
        )
        .api_route(
            "/batch_resolve",
            with_permission_check_post_with(
                batch_resolve,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::AdminRead.as_str(),
                    Permission::AdminManage.as_str(),
                    Permission::AuditLogRead.as_str(),
                ],
                |op| {
                    op.summary("Batch resolve admin IDs to names")
                        .tag("Admin Account")
                },
            ),
        )
        .with_state(state)
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(
        AdminOutput::from(admin),
        &t("Admin loaded"),
    ))
}

async fn create(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    AsyncContractJson(req): AsyncContractJson<CreateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::create(&state, &auth, req).await?;
    Ok(ApiResponse::success(
        AdminOutput::from(admin),
        &t("Admin created"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    AsyncContractJson(req): AsyncContractJson<UpdateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::update(&state, &auth, id, req).await?;
    Ok(ApiResponse::success(
        AdminOutput::from(admin),
        &t("Admin updated"),
    ))
}

async fn remove(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminDeleteOutput>, AppError> {
    workflow::remove(&state, &auth, id).await?;
    Ok(ApiResponse::success(
        AdminDeleteOutput { deleted: true },
        &t("Admin deleted"),
    ))
}

async fn batch_resolve(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    CleanJson(req): CleanJson<AdminBatchResolveInput>,
) -> Result<ApiResponse<AdminBatchResolveOutput>, AppError> {
    let parsed_ids: Vec<i64> = req
        .ids
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();
    let results = workflow::batch_resolve_names(&state, &parsed_ids).await?;
    let entries: Vec<AdminBatchResolveEntry> = results
        .into_iter()
        .map(|(id, username, name)| AdminBatchResolveEntry {
            id: id.into(),
            username,
            name,
        })
        .collect();
    Ok(ApiResponse::success(
        AdminBatchResolveOutput { entries },
        "ok",
    ))
}
