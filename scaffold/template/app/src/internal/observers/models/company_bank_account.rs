use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{
    CompanyBankAccountCreate, CompanyBankAccountRecord, CompanyBankAccountChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CompanyBankAccountCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(
    _event: &ModelEvent,
    _row: &CompanyBankAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[CompanyBankAccountRecord],
    _changes: &CompanyBankAccountChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CompanyBankAccountRecord,
    _new_row: &CompanyBankAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[CompanyBankAccountRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(
    _event: &ModelEvent,
    _row: &CompanyBankAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}
