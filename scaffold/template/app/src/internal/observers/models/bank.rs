use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{BankCreate, BankRecord, BankChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &BankCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> { Ok(()) }

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[BankRecord],
    _changes: &BankChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &BankRecord,
    _new_row: &BankRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[BankRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> { Ok(()) }
