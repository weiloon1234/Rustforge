use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    CompanyCryptoAccount, CompanyCryptoAccountQuery, CompanyCryptoAccountWithRelations,
    CryptoNetworkQuery, CryptoNetworkStatus,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::company_crypto_account::AdminCompanyCryptoAccountInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    id: i64,
) -> Result<CompanyCryptoAccountWithRelations, AppError> {
    CompanyCryptoAccountQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Company crypto account not found")))
}

async fn validate_crypto_network(state: &AppApiState, network_id: i64) -> Result<(), AppError> {
    let network = CryptoNetworkQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, network_id)
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
) -> Result<CompanyCryptoAccountWithRelations, AppError> {
    let network_id: i64 = req.crypto_network_id.into();
    validate_crypto_network(state, network_id).await?;

    let now = OffsetDateTime::now_utc();
    let row = CompanyCryptoAccount::new(DbConn::pool(&state.db), None)
        .insert()
        .set_crypto_network_id(network_id)
        .set_wallet_address(req.wallet_address)
        .set_conversion_rate(req.conversion_rate)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_created_at(now)
        .set_updated_at(now)
        .save()
        .await
        .map_err(AppError::from)?;

    detail(state, row.id).await
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminCompanyCryptoAccountInput,
) -> Result<CompanyCryptoAccountWithRelations, AppError> {
    let network_id: i64 = req.crypto_network_id.into();
    validate_crypto_network(state, network_id).await?;

    let affected = CompanyCryptoAccount::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_crypto_network_id(network_id)
        .set_wallet_address(req.wallet_address)
        .set_conversion_rate(req.conversion_rate)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_updated_at(OffsetDateTime::now_utc())
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company crypto account not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CompanyCryptoAccount::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company crypto account not found")));
    }

    Ok(())
}
