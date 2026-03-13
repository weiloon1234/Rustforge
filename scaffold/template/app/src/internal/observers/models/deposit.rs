use core_db::common::model_observer::ModelEvent;
use generated::models::{DepositCreateInput, DepositRow, DepositUpdateChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &DepositCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &DepositRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &DepositRow,
    _changes: &DepositUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &DepositRow,
    _new_row: &DepositRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &DepositRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &DepositRow) -> anyhow::Result<()> {
    Ok(())
}
