use core_db::common::model_observer::ModelEvent;
use generated::models::{BankCreate, BankRecord, BankChanges};

pub async fn creating(_event: &ModelEvent, _new_data: &BankCreate) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &BankRecord,
    _changes: &BankChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &BankRecord,
    _new_row: &BankRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> {
    Ok(())
}
