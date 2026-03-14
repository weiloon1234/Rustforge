use core_db::common::model_observer::ModelEvent;
use generated::models::{
    CompanyCryptoAccountCreate, CompanyCryptoAccountRecord, CompanyCryptoAccountChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CompanyCryptoAccountCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CompanyCryptoAccountRecord,
    _changes: &CompanyCryptoAccountChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CompanyCryptoAccountRecord,
    _new_row: &CompanyCryptoAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(
    _event: &ModelEvent,
    _row: &CompanyCryptoAccountRecord,
) -> anyhow::Result<()> {
    Ok(())
}
