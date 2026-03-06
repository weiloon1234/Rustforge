use axum::{
    extract::{Path, State},
    Json,
};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    extract::{validation::transform_validation_errors, AsyncValidate},
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_patch_with, with_permission_check_post_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};
use validator::Validate;

use crate::{
    contracts::api::v1::admin::account::{
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
    Json(req): Json<CreateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let req = validate_create_input(&state, req).await?;
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
    Json(req): Json<UpdateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let req = validate_update_input(&state, id, req).await?;
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

async fn validate_create_input(
    state: &AppApiState,
    req: CreateAdminInput,
) -> Result<CreateAdminInput, AppError> {
    let req = req.normalize();
    if let Err(e) = req.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    Ok(req)
}

async fn validate_update_input(
    state: &AppApiState,
    id: i64,
    req: UpdateAdminInput,
) -> Result<UpdateAdminInput, AppError> {
    let req = req.with_target_id(id).normalize();
    if let Err(e) = req.validate() {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    Ok(req)
}
