use core_db::common::model_observer::ModelEvent;
use generated::models::{
    IntroducerChangeCreateInput, IntroducerChangeRow, IntroducerChangeUpdateChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &IntroducerChangeCreateInput,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &IntroducerChangeRow) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &IntroducerChangeRow,
    _changes: &IntroducerChangeUpdateChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &IntroducerChangeRow,
    _new_row: &IntroducerChangeRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &IntroducerChangeRow,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &IntroducerChangeRow) -> anyhow::Result<()> {
    Ok(())
}
