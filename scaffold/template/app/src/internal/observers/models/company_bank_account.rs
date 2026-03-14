use core_db::common::model_observer::ModelEvent;
use generated::models::{
    CompanyBankAccountCreate, CompanyBankAccountRecord, CompanyBankAccountChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CompanyBankAccountCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CompanyBankAccountRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CompanyBankAccountRecord,
    _changes: &CompanyBankAccountChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CompanyBankAccountRecord,
    _new_row: &CompanyBankAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CompanyBankAccountRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CompanyBankAccountRecord) -> anyhow::Result<()> {
    Ok(())
}
