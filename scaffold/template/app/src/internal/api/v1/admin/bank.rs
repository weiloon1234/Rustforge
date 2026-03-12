use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::ContractJson,
    error::AppError,
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_post_with, with_permission_check_put_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::bank::{AdminBankInput, BankOutput},
    internal::{api::state::AppApiState, workflows::bank as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankManage.as_str()],
                |op| op.summary("Create bank").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/:id",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankRead.as_str(), Permission::BankManage.as_str()],
                |op| op.summary("Get bank detail").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/:id",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankManage.as_str()],
                |op| op.summary("Update bank").tag("Admin Banks"),
            ),
        )
        .api_route(
            "/:id",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::BankManage.as_str()],
                |op| op.summary("Delete bank").tag("Admin Banks"),
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminBankInput>,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let bank = workflow::create(&state, req).await?;

    Ok(ApiResponse::success(
        BankOutput {
            id: bank.id.into(),
            country_iso2: bank.country_iso2.clone(),
            name: bank.name.clone(),
            code: bank.code.clone(),
            logo_url: bank.logo_url.clone(),
            status: bank.status,
            sort_order: bank.sort_order,
            created_at: bank.created_at,
            updated_at: bank.updated_at,
        },
        &t("Bank created"),
    ))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let bank = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(
        BankOutput {
            id: bank.id.into(),
            country_iso2: bank.country_iso2.clone(),
            name: bank.name.clone(),
            code: bank.code.clone(),
            logo_url: bank.logo_url.clone(),
            status: bank.status,
            sort_order: bank.sort_order,
            created_at: bank.created_at,
            updated_at: bank.updated_at,
        },
        &t("Bank detail"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminBankInput>,
) -> Result<ApiResponse<BankOutput>, AppError> {
    let bank = workflow::update(&state, id, req).await?;

    Ok(ApiResponse::success(
        BankOutput {
            id: bank.id.into(),
            country_iso2: bank.country_iso2.clone(),
            name: bank.name.clone(),
            code: bank.code.clone(),
            logo_url: bank.logo_url.clone(),
            status: bank.status,
            sort_order: bank.sort_order,
            created_at: bank.created_at,
            updated_at: bank.updated_at,
        },
        &t("Bank updated"),
    ))
}

async fn delete(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    workflow::delete(&state, id).await?;

    Ok(ApiResponse::success(
        serde_json::json!({ "deleted": true }),
        &t("Bank deleted"),
    ))
}
