use core_db::common::model_observer::ModelEvent;
use generated::models::{UserCreateInput, UserRow, UserUpdateChanges};

pub async fn creating(_event: &ModelEvent, _new_data: &UserCreateInput) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &UserRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &UserRow,
    _changes: &UserUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &UserRow,
    _new_row: &UserRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &UserRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &UserRow) -> anyhow::Result<()> {
    Ok(())
}
