use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{AdminCreate, AdminRecord, AdminChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &AdminCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(_event: &ModelEvent, _row: &AdminRecord) -> anyhow::Result<()> { Ok(()) }

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[AdminRecord],
    _changes: &AdminChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &AdminRecord,
    _new_row: &AdminRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[AdminRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(_event: &ModelEvent, _row: &AdminRecord) -> anyhow::Result<()> { Ok(()) }
