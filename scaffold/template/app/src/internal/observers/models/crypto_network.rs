use core_db::common::model_observer::ModelEvent;
use generated::models::{CryptoNetworkCreate, CryptoNetworkRecord, CryptoNetworkChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CryptoNetworkCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CryptoNetworkRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CryptoNetworkRecord,
    _changes: &CryptoNetworkChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CryptoNetworkRecord,
    _new_row: &CryptoNetworkRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CryptoNetworkRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CryptoNetworkRecord) -> anyhow::Result<()> {
    Ok(())
}
