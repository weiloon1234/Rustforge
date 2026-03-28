use core_db::common::sql::{DbConn, Op, OrderDir};
use core_db::platform::attachments::types::AttachmentInput;
use core_i18n::t;
use core_web::error::AppError;
use generated::localized;
use generated::models::{
    CryptoNetworkCol, CryptoNetworkModel, CryptoNetworkRecord, CryptoNetworkStatus,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::crypto_network::AdminCryptoNetworkInput,
    internal::api::state::AppApiState,
};

pub async fn list_options(state: &AppApiState) -> Result<Vec<CryptoNetworkRecord>, AppError> {
    CryptoNetworkModel::query()
        .where_col(CryptoNetworkCol::STATUS, Op::Eq, CryptoNetworkStatus::Enabled)
        .order_by(CryptoNetworkCol::SORT_ORDER, OrderDir::Asc)
        .all(DbConn::pool(&state.db))
        .await
        .map_err(AppError::from)
}

pub async fn detail(state: &AppApiState, id: i64) -> Result<CryptoNetworkRecord, AppError> {
    CryptoNetworkModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Crypto network not found")))
}

pub async fn create(
    state: &AppApiState,
    req: AdminCryptoNetworkInput,
    logo: Option<AttachmentInput>,
) -> Result<CryptoNetworkRecord, AppError> {
    let now = OffsetDateTime::now_utc();
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let row = CryptoNetworkModel::create()
        .set(CryptoNetworkCol::NAME, req.name)?
        .set(CryptoNetworkCol::SYMBOL, req.symbol)?
        .set(CryptoNetworkCol::STATUS, req.status)?
        .set(CryptoNetworkCol::SORT_ORDER, req.sort_order.unwrap_or(0))?
        .set(CryptoNetworkCol::CREATED_AT, now)?
        .set(CryptoNetworkCol::UPDATED_AT, now)?
        .save(conn.clone())
        .await
        .map_err(AppError::from)?;

    if let Some(logo) = logo.as_ref() {
        localized::replace_single_attachment(
            conn.clone(),
            localized::CRYPTO_NETWORK_OWNER_TYPE,
            row.id,
            "logo",
            logo,
        )
        .await
        .map_err(AppError::from)?;
    }

    drop(conn);
    scope.commit().await.map_err(AppError::from)?;

    detail(state, row.id).await
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminCryptoNetworkInput,
    logo: Option<AttachmentInput>,
) -> Result<CryptoNetworkRecord, AppError> {
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let affected = CryptoNetworkModel::query()
        .where_col(CryptoNetworkCol::ID, Op::Eq, id)
        .patch()
        .assign(CryptoNetworkCol::NAME, req.name)?
        .assign(CryptoNetworkCol::SYMBOL, req.symbol)?
        .assign(CryptoNetworkCol::STATUS, req.status)?
        .assign(CryptoNetworkCol::SORT_ORDER, req.sort_order.unwrap_or(0))?
        .assign(CryptoNetworkCol::UPDATED_AT, OffsetDateTime::now_utc())?
        .save(conn.clone())
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Crypto network not found")));
    }

    if let Some(logo) = logo.as_ref() {
        localized::replace_single_attachment(
            conn.clone(),
            localized::CRYPTO_NETWORK_OWNER_TYPE,
            id,
            "logo",
            logo,
        )
        .await
        .map_err(AppError::from)?;
    }

    drop(conn);
    scope.commit().await.map_err(AppError::from)?;

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CryptoNetworkModel::query()
        .where_col(CryptoNetworkCol::ID, Op::Eq, id)
        .delete(DbConn::pool(&state.db))
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Crypto network not found")));
    }

    Ok(())
}
