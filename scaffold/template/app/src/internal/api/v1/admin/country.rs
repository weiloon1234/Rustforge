use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::ContractJson,
    error::AppError,
    openapi::{with_permission_check_patch_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::country::{
        AdminCountrySetDefaultOutput, AdminCountryStatusUpdateInput,
        AdminCountryStatusUpdateOutput,
    },
    internal::{api::state::AppApiState, workflows::country as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/{iso2}/status",
            with_permission_check_patch_with(
                update_status,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CountryManage.as_str()],
                |op| op.summary("Update country status").tag("Admin Country"),
            ),
        )
        .api_route(
            "/{iso2}/default",
            with_permission_check_patch_with(
                set_default,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CountryManage.as_str()],
                |op| op.summary("Set country as default").tag("Admin Country"),
            ),
        )
        .with_state(state)
}

async fn set_default(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(iso2): Path<String>,
) -> Result<ApiResponse<AdminCountrySetDefaultOutput>, AppError> {
    let country = workflow::set_default(&state, &iso2).await?;

    Ok(ApiResponse::success(
        AdminCountrySetDefaultOutput {
            iso2: country.iso2,
            is_default: country.is_default,
            updated_at: country.updated_at,
        },
        &t("Country default updated"),
    ))
}

async fn update_status(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(iso2): Path<String>,
    ContractJson(req): ContractJson<AdminCountryStatusUpdateInput>,
) -> Result<ApiResponse<AdminCountryStatusUpdateOutput>, AppError> {
    let country = workflow::update_status(&state, &iso2, &req.status).await?;

    Ok(ApiResponse::success(
        AdminCountryStatusUpdateOutput {
            iso2: country.iso2,
            status: country.status,
            updated_at: country.updated_at,
        },
        &t("Country status updated"),
    ))
}
