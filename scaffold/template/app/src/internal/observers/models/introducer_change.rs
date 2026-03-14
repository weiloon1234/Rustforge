use core_db::common::model_observer::ModelEvent;
use generated::models::{
    IntroducerChangeCreate, IntroducerChangeRecord, IntroducerChangeChanges,
};

pub async fn creating(
    _event: &ModelEvent,
    _new_data: &IntroducerChangeCreate,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn created(_event: &ModelEvent, _row: &IntroducerChangeRecord) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updating(
    _event: &ModelEvent,
    _old_row: &IntroducerChangeRecord,
    _changes: &IntroducerChangeChanges,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn updated(
    _event: &ModelEvent,
    _old_row: &IntroducerChangeRecord,
    _new_row: &IntroducerChangeRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleting(
    _event: &ModelEvent,
    _row: &IntroducerChangeRecord,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn deleted(_event: &ModelEvent, _row: &IntroducerChangeRecord) -> anyhow::Result<()> {
    Ok(())
}
