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
    contracts::api::v1::admin::crypto_network::{AdminCryptoNetworkInput, CryptoNetworkOutput},
    internal::{api::state::AppApiState, workflows::crypto_network as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Create crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::CryptoNetworkRead.as_str(),
                    Permission::CryptoNetworkManage.as_str(),
                ],
                |op| op.summary("Get crypto network detail").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Update crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CryptoNetworkManage.as_str()],
                |op| op.summary("Delete crypto network").tag("Admin Crypto Networks"),
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminCryptoNetworkInput>,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let network = workflow::create(&state, req).await?;

    Ok(ApiResponse::success(
        CryptoNetworkOutput {
            id: network.id.into(),
            name: network.name.clone(),
            symbol: network.symbol.clone(),
            logo_url: network.logo_url.clone(),
            status: network.status,
            sort_order: network.sort_order,
            created_at: network.created_at,
            updated_at: network.updated_at,
        },
        &t("Crypto network created"),
    ))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let network = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(
        CryptoNetworkOutput {
            id: network.id.into(),
            name: network.name.clone(),
            symbol: network.symbol.clone(),
            logo_url: network.logo_url.clone(),
            status: network.status,
            sort_order: network.sort_order,
            created_at: network.created_at,
            updated_at: network.updated_at,
        },
        &t("Crypto network detail"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminCryptoNetworkInput>,
) -> Result<ApiResponse<CryptoNetworkOutput>, AppError> {
    let network = workflow::update(&state, id, req).await?;

    Ok(ApiResponse::success(
        CryptoNetworkOutput {
            id: network.id.into(),
            name: network.name.clone(),
            symbol: network.symbol.clone(),
            logo_url: network.logo_url.clone(),
            status: network.status,
            sort_order: network.sort_order,
            created_at: network.created_at,
            updated_at: network.updated_at,
        },
        &t("Crypto network updated"),
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
        &t("Crypto network deleted"),
    ))
}
