use core_db::common::model_observer::ModelEvent;
use generated::models::{BankCreateInput, BankRow, BankUpdateChanges};

pub async fn creating(_event: &ModelEvent, _new_data: &BankCreateInput) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &BankRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &BankRow,
    _changes: &BankUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &BankRow,
    _new_row: &BankRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &BankRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &BankRow) -> anyhow::Result<()> {
    Ok(())
}
