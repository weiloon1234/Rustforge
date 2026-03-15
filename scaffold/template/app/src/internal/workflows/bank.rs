use core_db::common::sql::{DbConn, Op};
use core_db::platform::attachments::types::AttachmentInput;
use core_i18n::t;
use core_web::error::AppError;
use generated::localized;
use generated::models::{BankCol, BankModel, BankRecord, CountryCol, CountryModel};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::bank::AdminBankInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<BankRecord, AppError> {
    BankModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Bank not found")))
}

pub async fn create(
    state: &AppApiState,
    req: AdminBankInput,
    logo: Option<AttachmentInput>,
) -> Result<BankRecord, AppError> {
    let country_exists = CountryModel::query(DbConn::pool(&state.db))
        .where_col(CountryCol::ISO2, Op::Eq, req.country_iso2.clone())
        .count()
        .await
        .map_err(AppError::from)? > 0;

    if !country_exists {
        return Err(AppError::BadRequest(t("Country not found")));
    }

    let now = OffsetDateTime::now_utc();
    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let row = BankModel::create(conn.clone())
        .set(BankCol::COUNTRY_ISO2, req.country_iso2)?
        .set(BankCol::NAME, req.name)?
        .set(BankCol::CODE, req.code)?
        .set(BankCol::STATUS, req.status)?
        .set(BankCol::SORT_ORDER, req.sort_order.unwrap_or(0))?
        .set(BankCol::CREATED_AT, now)?
        .set(BankCol::UPDATED_AT, now)?
        .save()
        .await
        .map_err(AppError::from)?;

    if let Some(logo) = logo.as_ref() {
        localized::replace_single_attachment(conn, localized::BANK_OWNER_TYPE, row.id, "logo", logo)
            .await
            .map_err(AppError::from)?;
    }

    scope.commit().await.map_err(AppError::from)?;

    detail(state, row.id).await
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminBankInput,
    logo: Option<AttachmentInput>,
) -> Result<BankRecord, AppError> {
    let country_exists = CountryModel::query(DbConn::pool(&state.db))
        .where_col(CountryCol::ISO2, Op::Eq, req.country_iso2.clone())
        .count()
        .await
        .map_err(AppError::from)? > 0;

    if !country_exists {
        return Err(AppError::BadRequest(t("Country not found")));
    }

    let scope = DbConn::pool(&state.db)
        .begin_scope()
        .await
        .map_err(AppError::from)?;
    let conn = scope.conn();

    let affected = BankModel::query(conn.clone())
        .where_col(BankCol::ID, Op::Eq, id)
        .patch()
        .assign(BankCol::COUNTRY_ISO2, req.country_iso2)?
        .assign(BankCol::NAME, req.name)?
        .assign(BankCol::CODE, req.code)?
        .assign(BankCol::STATUS, req.status)?
        .assign(BankCol::SORT_ORDER, req.sort_order.unwrap_or(0))?
        .assign(BankCol::UPDATED_AT, OffsetDateTime::now_utc())?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Bank not found")));
    }

    if let Some(logo) = logo.as_ref() {
        localized::replace_single_attachment(conn, localized::BANK_OWNER_TYPE, id, "logo", logo)
            .await
            .map_err(AppError::from)?;
    }

    scope.commit().await.map_err(AppError::from)?;

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = BankModel::query(DbConn::pool(&state.db))
        .where_col(BankCol::ID, Op::Eq, id)
        .delete()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Bank not found")));
    }

    Ok(())
}
