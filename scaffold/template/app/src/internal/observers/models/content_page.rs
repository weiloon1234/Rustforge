use core_db::common::model_observer::ModelEvent;
use generated::models::{ContentPageCreateInput, ContentPageRow, ContentPageUpdateChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &ContentPageCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &ContentPageRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &ContentPageRow,
    _changes: &ContentPageUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &ContentPageRow,
    _new_row: &ContentPageRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &ContentPageRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &ContentPageRow) -> anyhow::Result<()> {
    Ok(())
}
