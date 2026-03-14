use core_db::common::model_observer::ModelEvent;
use generated::models::{
    UserCreditTransactionCreate, UserCreditTransactionRecord, UserCreditTransactionChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &UserCreditTransactionCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &UserCreditTransactionRecord,
    _changes: &UserCreditTransactionChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &UserCreditTransactionRecord,
    _new_row: &UserCreditTransactionRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRecord,
) -> anyhow::Result<()> {
    Ok(())
}
