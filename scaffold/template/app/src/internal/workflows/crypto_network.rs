use core_db::common::sql::{DbConn, Op};
use core_db::platform::attachments::types::AttachmentInput;
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{CryptoNetwork, CryptoNetworkQuery, CryptoNetworkWithRelations};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::crypto_network::AdminCryptoNetworkInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<CryptoNetworkWithRelations, AppError> {
    CryptoNetworkQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Crypto network not found")))
}

pub async fn create(
    state: &AppApiState,
    req: AdminCryptoNetworkInput,
    logo: Option<AttachmentInput>,
) -> Result<CryptoNetworkWithRelations, AppError> {
    let now = OffsetDateTime::now_utc();
    let mut insert = CryptoNetwork::new(DbConn::pool(&state.db), None)
        .insert()
        .set_name(req.name)
        .set_symbol(req.symbol)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_created_at(now)
        .set_updated_at(now);

    if let Some(logo) = logo {
        insert = insert.set_logo(logo);
    }

    let row = insert.save().await.map_err(AppError::from)?;

    detail(state, row.id).await
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminCryptoNetworkInput,
    logo: Option<AttachmentInput>,
) -> Result<CryptoNetworkWithRelations, AppError> {
    let mut update = CryptoNetwork::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_name(req.name)
        .set_symbol(req.symbol)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_updated_at(OffsetDateTime::now_utc());

    if let Some(logo) = logo {
        update = update.set_logo(logo);
    }

    let affected = update.save().await.map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Crypto network not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CryptoNetwork::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Crypto network not found")));
    }

    Ok(())
}
