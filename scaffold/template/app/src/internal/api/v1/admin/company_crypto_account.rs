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
    contracts::api::v1::admin::company_crypto_account::{
        AdminCompanyCryptoAccountInput, CompanyCryptoAccountOutput,
    },
    internal::{api::state::AppApiState, workflows::company_crypto_account as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyCryptoAccountManage.as_str()],
                |op| {
                    op.summary("Create company crypto account")
                        .tag("Admin Company Crypto Accounts")
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
                    Permission::CompanyCryptoAccountRead.as_str(),
                    Permission::CompanyCryptoAccountManage.as_str(),
                ],
                |op| {
                    op.summary("Get company crypto account detail")
                        .tag("Admin Company Crypto Accounts")
                },
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyCryptoAccountManage.as_str()],
                |op| {
                    op.summary("Update company crypto account")
                        .tag("Admin Company Crypto Accounts")
                },
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyCryptoAccountManage.as_str()],
                |op| {
                    op.summary("Delete company crypto account")
                        .tag("Admin Company Crypto Accounts")
                },
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminCompanyCryptoAccountInput>,
) -> Result<ApiResponse<CompanyCryptoAccountOutput>, AppError> {
    let account = workflow::create(&state, req).await?;

    Ok(ApiResponse::success(
        CompanyCryptoAccountOutput {
            id: account.id.into(),
            crypto_network_id: account.crypto_network_id.into(),
            crypto_network_name: account.crypto_network.as_ref().map(|n| n.name.clone()),
            wallet_address: account.wallet_address.clone(),
            conversion_rate: account.conversion_rate,
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company crypto account created"),
    ))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<CompanyCryptoAccountOutput>, AppError> {
    let account = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(
        CompanyCryptoAccountOutput {
            id: account.id.into(),
            crypto_network_id: account.crypto_network_id.into(),
            crypto_network_name: account.crypto_network.as_ref().map(|n| n.name.clone()),
            wallet_address: account.wallet_address.clone(),
            conversion_rate: account.conversion_rate,
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company crypto account detail"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminCompanyCryptoAccountInput>,
) -> Result<ApiResponse<CompanyCryptoAccountOutput>, AppError> {
    let account = workflow::update(&state, id, req).await?;

    Ok(ApiResponse::success(
        CompanyCryptoAccountOutput {
            id: account.id.into(),
            crypto_network_id: account.crypto_network_id.into(),
            crypto_network_name: account.crypto_network.as_ref().map(|n| n.name.clone()),
            wallet_address: account.wallet_address.clone(),
            conversion_rate: account.conversion_rate,
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company crypto account updated"),
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
        &t("Company crypto account deleted"),
    ))
}
