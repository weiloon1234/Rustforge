use core_db::common::model_observer::ModelEvent;
use generated::models::{WithdrawalCreate, WithdrawalRecord, WithdrawalChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &WithdrawalCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &WithdrawalRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &WithdrawalRecord,
    _changes: &WithdrawalChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &WithdrawalRecord,
    _new_row: &WithdrawalRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &WithdrawalRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &WithdrawalRecord) -> anyhow::Result<()> {
    Ok(())
}
