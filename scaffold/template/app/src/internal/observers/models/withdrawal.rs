use core_db::common::model_observer::ModelEvent;
use generated::models::{WithdrawalCreateInput, WithdrawalRow, WithdrawalUpdateChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &WithdrawalCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &WithdrawalRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &WithdrawalRow,
    _changes: &WithdrawalUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &WithdrawalRow,
    _new_row: &WithdrawalRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &WithdrawalRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &WithdrawalRow) -> anyhow::Result<()> {
    Ok(())
}
