use core_db::common::model_observer::ModelEvent;
use generated::models::{CountryCreateInput, CountryRow, CountryUpdateChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CountryCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CountryRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CountryRow,
    _changes: &CountryUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CountryRow,
    _new_row: &CountryRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CountryRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CountryRow) -> anyhow::Result<()> {
    Ok(())
}
