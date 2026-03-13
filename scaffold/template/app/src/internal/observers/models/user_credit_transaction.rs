use core_db::common::model_observer::ModelEvent;
use generated::models::{
    UserCreditTransactionCreateInput, UserCreditTransactionRow, UserCreditTransactionUpdateChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &UserCreditTransactionCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &UserCreditTransactionRow,
    _changes: &UserCreditTransactionUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &UserCreditTransactionRow,
    _new_row: &UserCreditTransactionRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(
    _event: &ModelEvent,
    _row: &UserCreditTransactionRow,
) -> anyhow::Result<()> {
    Ok(())
}
