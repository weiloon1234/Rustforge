use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    extract::{validation::transform_validation_errors, AsyncValidate, CleanJson},
    openapi::{
        with_permission_check_get_with, with_permission_check_patch_with,
        with_permission_check_post_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, models::UserBanStatus, permissions::Permission};
use validator::Validate;

use crate::{
    contracts::api::v1::admin::user::{
        BatchResolveEntry, BatchResolveInput, BatchResolveOutput, CreateUserInput, UpdateUserInput,
        UserBanInput, UserBanOutput, UserManageOutput,
    },
    internal::{api::state::AppApiState, workflows::user_manage as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserManage.as_str()],
                |op| op.summary("Create user").tag("Admin User Management"),
            ),
        )
        .api_route(
            "/batch_resolve",
            with_permission_check_post_with(
                batch_resolve,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::UserRead.as_str(),
                    Permission::UserManage.as_str(),
                ],
                |op| {
                    op.summary("Batch resolve user IDs to usernames")
                        .tag("Admin User Management")
                },
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::UserRead.as_str(),
                    Permission::UserManage.as_str(),
                ],
                |op| op.summary("Get user detail").tag("Admin User Management"),
            )
            .merge(with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserManage.as_str()],
                |op| op.summary("Update user").tag("Admin User Management"),
            )),
        )
        .api_route(
            "/{id}/ban",
            with_permission_check_patch_with(
                set_ban,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserManage.as_str()],
                |op| {
                    op.summary("Ban or unban user")
                        .tag("Admin User Management")
                },
            ),
        )
        .with_state(state)
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<UserManageOutput>, AppError> {
    let user = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(
        UserManageOutput::from(user),
        &t("User loaded"),
    ))
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    CleanJson(req): CleanJson<CreateUserInput>,
) -> Result<ApiResponse<UserManageOutput>, AppError> {
    let req = validate_create_input(&state, req).await?;
    let user = workflow::create(&state, req).await?;
    Ok(ApiResponse::success(
        UserManageOutput::from(user),
        &t("User created"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    CleanJson(req): CleanJson<UpdateUserInput>,
) -> Result<ApiResponse<UserManageOutput>, AppError> {
    let req = validate_update_input(&state, req).await?;
    let user = workflow::update(&state, id, req).await?;
    Ok(ApiResponse::success(
        UserManageOutput::from(user),
        &t("User updated"),
    ))
}

async fn set_ban(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    CleanJson(req): CleanJson<UserBanInput>,
) -> Result<ApiResponse<UserBanOutput>, AppError> {
    let _user = workflow::set_ban(&state, id, req.ban).await?;
    let banned = matches!(req.ban, UserBanStatus::Yes);
    let message = if banned {
        t("User banned")
    } else {
        t("User unbanned")
    };
    Ok(ApiResponse::success(UserBanOutput { banned }, &message))
}

async fn batch_resolve(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    CleanJson(req): CleanJson<BatchResolveInput>,
) -> Result<ApiResponse<BatchResolveOutput>, AppError> {
    let parsed_ids: Vec<i64> = req
        .ids
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();
    let results = workflow::batch_resolve_usernames(&state, &parsed_ids).await?;
    let entries: Vec<BatchResolveEntry> = results
        .into_iter()
        .map(|(id, username, name)| BatchResolveEntry {
            id: id.into(),
            username,
            name,
        })
        .collect();
    Ok(ApiResponse::success(BatchResolveOutput { entries }, "ok"))
}

async fn validate_create_input(
    state: &AppApiState,
    req: CreateUserInput,
) -> Result<CreateUserInput, AppError> {
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
    req: UpdateUserInput,
) -> Result<UpdateUserInput, AppError> {
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
