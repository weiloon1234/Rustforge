use core_db::common::model_observer::ModelEvent;
use generated::models::{
    CompanyCryptoAccountCreateInput, CompanyCryptoAccountRow, CompanyCryptoAccountUpdateChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CompanyCryptoAccountCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CompanyCryptoAccountRow,
    _changes: &CompanyCryptoAccountUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CompanyCryptoAccountRow,
    _new_row: &CompanyCryptoAccountRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRow,
) -> anyhow::Result<()> {
    Ok(())
}
