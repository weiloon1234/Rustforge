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
    contracts::api::v1::admin::company_bank_account::{
        AdminCompanyBankAccountInput, CompanyBankAccountOutput,
    },
    internal::{api::state::AppApiState, workflows::company_bank_account as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyBankAccountManage.as_str()],
                |op| op.summary("Create company bank account").tag("Admin Company Bank Accounts"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [
                    Permission::CompanyBankAccountRead.as_str(),
                    Permission::CompanyBankAccountManage.as_str(),
                ],
                |op| op.summary("Get company bank account detail").tag("Admin Company Bank Accounts"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_put_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyBankAccountManage.as_str()],
                |op| op.summary("Update company bank account").tag("Admin Company Bank Accounts"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_delete_with(
                delete,
                AdminGuard,
                PermissionMode::Any,
                [Permission::CompanyBankAccountManage.as_str()],
                |op| op.summary("Delete company bank account").tag("Admin Company Bank Accounts"),
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    ContractJson(req): ContractJson<AdminCompanyBankAccountInput>,
) -> Result<ApiResponse<CompanyBankAccountOutput>, AppError> {
    let account = workflow::create(&state, req).await?;

    Ok(ApiResponse::success(
        CompanyBankAccountOutput {
            id: account.id.into(),
            bank_id: account.bank_id.into(),
            bank_name: account.bank.as_ref().map(|b| b.name.clone()),
            account_name: account.account_name.clone(),
            account_number: account.account_number.clone(),
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company bank account created"),
    ))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<CompanyBankAccountOutput>, AppError> {
    let account = workflow::detail(&state, id).await?;

    Ok(ApiResponse::success(
        CompanyBankAccountOutput {
            id: account.id.into(),
            bank_id: account.bank_id.into(),
            bank_name: account.bank.as_ref().map(|b| b.name.clone()),
            account_name: account.account_name.clone(),
            account_number: account.account_number.clone(),
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company bank account detail"),
    ))
}

async fn update(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    ContractJson(req): ContractJson<AdminCompanyBankAccountInput>,
) -> Result<ApiResponse<CompanyBankAccountOutput>, AppError> {
    let account = workflow::update(&state, id, req).await?;

    Ok(ApiResponse::success(
        CompanyBankAccountOutput {
            id: account.id.into(),
            bank_id: account.bank_id.into(),
            bank_name: account.bank.as_ref().map(|b| b.name.clone()),
            account_name: account.account_name.clone(),
            account_number: account.account_number.clone(),
            status: account.status,
            sort_order: account.sort_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        },
        &t("Company bank account updated"),
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
        &t("Company bank account deleted"),
    ))
}
