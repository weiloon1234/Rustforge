use axum::extract::State;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    contracts::ContractJson,
    error::AppError,
    openapi::{
        aide::axum::routing::{get_with, post_with},
        ApiRouter,
    },
    response::ApiResponse,
};
use generated::guards::UserGuard;

use crate::{
    contracts::api::v1::user::wallet::{
        UserWithdrawalHistoryQuery, UserWithdrawalHistoryResponse,
        UserWithdrawalInput, UserWithdrawalOutput, WalletLedgerQuery, WalletLedgerResponse,
        WithdrawalConfigResponse,
    },
    internal::{api::state::AppApiState, workflows::wallet as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/ledger",
            get_with(ledger, |op| {
                op.summary("Get user credit ledger").tag("Wallet")
            }),
        )
        .api_route(
            "/crypto-networks",
            get_with(crypto_networks, |op| {
                op.summary("Get enabled crypto networks").tag("Wallet")
            }),
        )
        .api_route(
            "/withdrawal",
            post_with(create_withdrawal, |op| {
                op.summary("Create withdrawal request").tag("Wallet")
            }),
        )
        .api_route(
            "/withdrawal/history",
            get_with(withdrawal_history, |op| {
                op.summary("Get withdrawal history").tag("Wallet")
            }),
        )
        .with_state(state)
}

async fn ledger(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    axum::extract::Query(query): axum::extract::Query<WalletLedgerQuery>,
) -> Result<ApiResponse<WalletLedgerResponse>, AppError> {
    let result = workflow::get_ledger(
        &state,
        auth.user.id,
        query.limit,
        query.cursor.as_deref(),
    )
    .await?;
    Ok(ApiResponse::success(result, &t("Ledger loaded")))
}

async fn crypto_networks(
    State(state): State<AppApiState>,
    _auth: AuthUser<UserGuard>,
) -> Result<ApiResponse<WithdrawalConfigResponse>, AppError> {
    let networks = workflow::get_crypto_networks(&state).await?;
    Ok(ApiResponse::success(
        WithdrawalConfigResponse {
            networks,
            fee_percentage: state.withdrawal_fee_config.fee_percentage.into(),
            min_amount: state.withdrawal_fee_config.min_amount.into(),
        },
        &t("Crypto networks loaded"),
    ))
}

async fn create_withdrawal(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    ContractJson(req): ContractJson<UserWithdrawalInput>,
) -> Result<ApiResponse<UserWithdrawalOutput>, AppError> {
    let result = workflow::create_withdrawal(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(result, &t("Withdrawal request submitted")))
}

async fn withdrawal_history(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    axum::extract::Query(query): axum::extract::Query<UserWithdrawalHistoryQuery>,
) -> Result<ApiResponse<UserWithdrawalHistoryResponse>, AppError> {
    let result = workflow::get_withdrawal_history(
        &state,
        auth.user.id,
        query.limit,
        query.cursor.as_deref(),
    )
    .await?;
    Ok(ApiResponse::success(result, &t("Withdrawal history loaded")))
}
