use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{ContentPageCreate, ContentPageRecord, ContentPageChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &ContentPageCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(_event: &ModelEvent, _row: &ContentPageRecord) -> anyhow::Result<()> { Ok(()) }

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[ContentPageRecord],
    _changes: &ContentPageChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &ContentPageRecord,
    _new_row: &ContentPageRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[ContentPageRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(_event: &ModelEvent, _row: &ContentPageRecord) -> anyhow::Result<()> { Ok(()) }
