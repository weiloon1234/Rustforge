use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    BankCol, BankModel, BankStatus, CompanyBankAccountCol, CompanyBankAccountModel,
    CompanyBankAccountRecord,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::company_bank_account::AdminCompanyBankAccountInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    id: i64,
) -> Result<CompanyBankAccountRecord, AppError> {
    CompanyBankAccountModel::find(DbConn::pool(&state.db), id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Company bank account not found")))
}

async fn validate_bank(state: &AppApiState, bank_id: i64) -> Result<(), AppError> {
    let bank = BankModel::query(DbConn::pool(&state.db))
        .where_col(BankCol::ID, Op::Eq, bank_id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::BadRequest(t("Bank not found")))?;

    if bank.status != BankStatus::Enabled {
        return Err(AppError::BadRequest(t("Bank is not enabled")));
    }

    Ok(())
}

pub async fn create(
    state: &AppApiState,
    req: AdminCompanyBankAccountInput,
) -> Result<CompanyBankAccountRecord, AppError> {
    let bank_id: i64 = req.bank_id.into();
    validate_bank(state, bank_id).await?;

    let now = OffsetDateTime::now_utc();
    let row = CompanyBankAccountModel::create(DbConn::pool(&state.db))
        .set(CompanyBankAccountCol::BANK_ID, bank_id)
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::ACCOUNT_NAME, req.account_name)
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::ACCOUNT_NUMBER, req.account_number)
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::STATUS, req.status)
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::SORT_ORDER, req.sort_order.unwrap_or(0))
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::CREATED_AT, now)
        .map_err(AppError::from)?
        .set(CompanyBankAccountCol::UPDATED_AT, now)
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    Ok(row)
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminCompanyBankAccountInput,
) -> Result<CompanyBankAccountRecord, AppError> {
    let bank_id: i64 = req.bank_id.into();
    validate_bank(state, bank_id).await?;

    let affected = CompanyBankAccountModel::query(DbConn::pool(&state.db))
        .where_col(CompanyBankAccountCol::ID, Op::Eq, id)
        .patch()
        .assign(CompanyBankAccountCol::BANK_ID, bank_id)
        .map_err(AppError::from)?
        .assign(CompanyBankAccountCol::ACCOUNT_NAME, req.account_name)
        .map_err(AppError::from)?
        .assign(CompanyBankAccountCol::ACCOUNT_NUMBER, req.account_number)
        .map_err(AppError::from)?
        .assign(CompanyBankAccountCol::STATUS, req.status)
        .map_err(AppError::from)?
        .assign(CompanyBankAccountCol::SORT_ORDER, req.sort_order.unwrap_or(0))
        .map_err(AppError::from)?
        .assign(CompanyBankAccountCol::UPDATED_AT, OffsetDateTime::now_utc())
        .map_err(AppError::from)?
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company bank account not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CompanyBankAccountModel::query(DbConn::pool(&state.db))
        .where_col(CompanyBankAccountCol::ID, Op::Eq, id)
        .delete()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company bank account not found")));
    }

    Ok(())
}
