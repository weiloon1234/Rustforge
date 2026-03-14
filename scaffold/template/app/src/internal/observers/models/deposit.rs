use core_db::common::model_observer::ModelEvent;
use generated::models::{DepositCreate, DepositRecord, DepositChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &DepositCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &DepositRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &DepositRecord,
    _changes: &DepositChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &DepositRecord,
    _new_row: &DepositRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &DepositRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &DepositRecord) -> anyhow::Result<()> {
    Ok(())
}
