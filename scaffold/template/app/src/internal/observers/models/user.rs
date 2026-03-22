use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{UserCreate, UserRecord, UserChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &UserCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(_event: &ModelEvent, _row: &UserRecord) -> anyhow::Result<()> { Ok(()) }

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[UserRecord],
    _changes: &UserChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &UserRecord,
    _new_row: &UserRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[UserRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(_event: &ModelEvent, _row: &UserRecord) -> anyhow::Result<()> { Ok(()) }
