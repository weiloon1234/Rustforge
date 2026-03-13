use core_db::common::model_observer::ModelEvent;
use generated::models::{
    CompanyBankAccountCreateInput, CompanyBankAccountRow, CompanyBankAccountUpdateChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CompanyBankAccountCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CompanyBankAccountRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CompanyBankAccountRow,
    _changes: &CompanyBankAccountUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CompanyBankAccountRow,
    _new_row: &CompanyBankAccountRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CompanyBankAccountRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CompanyBankAccountRow) -> anyhow::Result<()> {
    Ok(())
}
