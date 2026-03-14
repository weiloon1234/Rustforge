use core_db::common::model_observer::ModelEvent;
use generated::models::{ContentPageCreate, ContentPageRecord, ContentPageChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &ContentPageCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &ContentPageRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &ContentPageRecord,
    _changes: &ContentPageChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &ContentPageRecord,
    _new_row: &ContentPageRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &ContentPageRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &ContentPageRecord) -> anyhow::Result<()> {
    Ok(())
}
