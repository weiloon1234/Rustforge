use core_db::common::sql::{DbConn, Op};
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{
    BankQuery, BankStatus, CompanyBankAccount, CompanyBankAccountQuery,
    CompanyBankAccountWithRelations,
};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::company_bank_account::AdminCompanyBankAccountInput,
    internal::api::state::AppApiState,
};

pub async fn detail(
    state: &AppApiState,
    id: i64,
) -> Result<CompanyBankAccountWithRelations, AppError> {
    CompanyBankAccountQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Company bank account not found")))
}

async fn validate_bank(state: &AppApiState, bank_id: i64) -> Result<(), AppError> {
    let bank = BankQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, bank_id)
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
) -> Result<CompanyBankAccountWithRelations, AppError> {
    let bank_id: i64 = req.bank_id.into();
    validate_bank(state, bank_id).await?;

    let now = OffsetDateTime::now_utc();
    let row = CompanyBankAccount::new(DbConn::pool(&state.db), None)
        .insert()
        .set_bank_id(bank_id)
        .set_account_name(req.account_name)
        .set_account_number(req.account_number)
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
    req: AdminCompanyBankAccountInput,
) -> Result<CompanyBankAccountWithRelations, AppError> {
    let bank_id: i64 = req.bank_id.into();
    validate_bank(state, bank_id).await?;

    let affected = CompanyBankAccount::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_bank_id(bank_id)
        .set_account_name(req.account_name)
        .set_account_number(req.account_number)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_updated_at(OffsetDateTime::now_utc())
        .save()
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company bank account not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = CompanyBankAccount::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Company bank account not found")));
    }

    Ok(())
}
