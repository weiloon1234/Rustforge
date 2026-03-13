use core_db::common::model_observer::ModelEvent;
use generated::models::{AdminCreateInput, AdminRow, AdminUpdateChanges};

pub async fn creating(_event: &ModelEvent, _new_data: &AdminCreateInput) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &AdminRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &AdminRow,
    _changes: &AdminUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &AdminRow,
    _new_row: &AdminRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &AdminRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &AdminRow) -> anyhow::Result<()> {
    Ok(())
}
