use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{CountryCreate, CountryRecord, CountryChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CountryCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn created(_event: &ModelEvent, _row: &CountryRecord) -> anyhow::Result<()> { Ok(()) }

pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[CountryRecord],
    _changes: &CountryChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CountryRecord,
    _new_row: &CountryRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[CountryRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

pub async fn deleted(_event: &ModelEvent, _row: &CountryRecord) -> anyhow::Result<()> { Ok(()) }
