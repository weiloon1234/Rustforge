use core_db::common::model_observer::ModelEvent;
use generated::models::{CountryCreate, CountryRecord, CountryChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CountryCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CountryRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CountryRecord,
    _changes: &CountryChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CountryRecord,
    _new_row: &CountryRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CountryRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CountryRecord) -> anyhow::Result<()> {
    Ok(())
}
