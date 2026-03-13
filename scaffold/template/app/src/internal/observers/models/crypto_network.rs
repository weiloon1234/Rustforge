use core_db::common::model_observer::ModelEvent;
use generated::models::{CryptoNetworkCreateInput, CryptoNetworkRow, CryptoNetworkUpdateChanges};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &CryptoNetworkCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &CryptoNetworkRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &CryptoNetworkRow,
    _changes: &CryptoNetworkUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &CryptoNetworkRow,
    _new_row: &CryptoNetworkRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(_event: &ModelEvent, _row: &CryptoNetworkRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &CryptoNetworkRow) -> anyhow::Result<()> {
    Ok(())
}
