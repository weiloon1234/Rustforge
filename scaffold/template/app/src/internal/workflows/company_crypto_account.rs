use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CompanyCryptoAccountCol, CompanyCryptoAccountModel, CompanyCryptoAccountRecord,
    CryptoNetworkCol, CryptoNetworkModel, CryptoNetworkStatus,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::company_crypto_account::AdminCompanyCryptoAccountInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    id: i64,
) -> Result<CompanyCryptoAccountRecord, AppError> {
    CompanyCryptoAccountModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Company crypto account not found")))
}

async fn validate_crypto_network(state: &AppApiState, network_id: i64) -> Result<(), AppError> {
    let network = CryptoNetworkModel::query(DbConn::pool(&state.db))
        .where_col(CryptoNetworkCol::ID, Op::Eq, network_id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::BadRequest(t("Crypto network not found")))?;

    if network.status != CryptoNetworkStatus::Enabled {
        return Err(AppError::BadRequest(t("Crypto network is not enabled")));
    }

    Ok(())
}

pub async fn create(
    state: &AppApiState,
    req: AdminCompanyCryptoAccountInput,
) -> Result<CompanyCryptoAccountRecord, AppError> {
    let network_id: i64 = req.crypto_network_id.into();
    validate_crypto_network(state, network_id).await?;

    let now = OffsetDateTime::now_utc();
    let row = CompanyCryptoAccountModel::create(DbConn::pool(&state.db))
        .set(CompanyCryptoAccountCol::CRYPTO_NETWORK_ID, network_id)
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::WALLET_ADDRESS, req.wallet_address)
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::CONVERSION_RATE, req.conversion_rate)
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::STATUS, req.status)
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::SORT_ORDER, req.sort_order.unwrap_or(0))
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::CREATED_AT, now)
        .map_err(AppError::from)?
        .set(CompanyCryptoAccountCol::UPDATED_AT, now)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    Ok(row)
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminCompanyCryptoAccountInput,
) -> Result<CompanyCryptoAccountRecord, AppError> {
    let network_id: i64 = req.crypto_network_id.into();
    validate_crypto_network(state, network_id).await?;

    let affected = CompanyCryptoAccountModel::query(DbConn::pool(&state.db))
        .where_col(CompanyCryptoAccountCol::ID, Op::Eq, id)
        .patch()
        .assign(CompanyCryptoAccountCol::CRYPTO_NETWORK_ID, network_id)
        .map_err(AppError::from)?
        .assign(CompanyCryptoAccountCol::WALLET_ADDRESS, req.wallet_address)
        .map_err(AppError::from)?
        .assign(CompanyCryptoAccountCol::CONVERSION_RATE, req.conversion_rate)
        .map_err(AppError::from)?
        .assign(CompanyCryptoAccountCol::STATUS, req.status)
        .map_err(AppError::from)?
        .assign(CompanyCryptoAccountCol::SORT_ORDER, req.sort_order.unwrap_or(0))
        .map_err(AppError::from)?
        .assign(CompanyCryptoAccountCol::UPDATED_AT, OffsetDateTime::now_utc())
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company crypto account not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CompanyCryptoAccountModel::query(DbConn::pool(&state.db))
        .where_col(CompanyCryptoAccountCol::ID, Op::Eq, id)
        .delete()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company crypto account not found")));
    }

    Ok(())
}
