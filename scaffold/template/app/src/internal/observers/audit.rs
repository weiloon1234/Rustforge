use core_db::common::model_observer::ModelEvent;
use core_db::common::sql::{generate_snowflake_i64, DbConn};
use generated::models::{AuditAction, AuditLog};

pub async fn created(
    db: &sqlx::PgPool,
    admin_id: i64,
    event: &ModelEvent,
    new_data: &serde_json::Value,
) -> anyhow::Result<()> {
    write_log_raw(
        db,
        admin_id,
        AuditAction::Create,
        event.table,
        event.record_key.as_deref().unwrap_or_default(),
        None,
        Some(new_data.clone()),
    )
    .await;
    Ok(())
}

pub async fn updated(
    db: &sqlx::PgPool,
    admin_id: i64,
    event: &ModelEvent,
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
) -> anyhow::Result<()> {
    let (old_diff, new_diff) = compute_diff(old_data, new_data);
    if old_diff.is_none() && new_diff.is_none() {
        return Ok(());
    }

    write_log_raw(
        db,
        admin_id,
        AuditAction::Update,
        event.table,
        event.record_key.as_deref().unwrap_or_default(),
        old_diff,
        new_diff,
    )
    .await;
    Ok(())
}

pub async fn deleted(
    db: &sqlx::PgPool,
    admin_id: i64,
    event: &ModelEvent,
    old_data: &serde_json::Value,
) -> anyhow::Result<()> {
    write_log_raw(
        db,
        admin_id,
        AuditAction::Delete,
        event.table,
        event.record_key.as_deref().unwrap_or_default(),
        Some(old_data.clone()),
        None,
    )
    .await;
    Ok(())
}

fn compute_diff(
    old_data: &serde_json::Value,
    new_data: &serde_json::Value,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    if let (serde_json::Value::Object(old_map), serde_json::Value::Object(new_map)) =
        (old_data, new_data)
    {
        let mut old_changes = serde_json::Map::new();
        let mut new_changes = serde_json::Map::new();

        for (key, new_val) in new_map {
            if let Some(old_val) = old_map.get(key) {
                if old_val != new_val {
                    old_changes.insert(key.clone(), old_val.clone());
                    new_changes.insert(key.clone(), new_val.clone());
                }
            }
        }

        if old_changes.is_empty() {
            return (None, None);
        }

        (
            Some(serde_json::Value::Object(old_changes)),
            Some(serde_json::Value::Object(new_changes)),
        )
    } else {
        (Some(old_data.clone()), Some(new_data.clone()))
    }
}

async fn write_log_raw(
    db: &sqlx::PgPool,
    admin_id: i64,
    action: AuditAction,
    table_name: &str,
    record_key: &str,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
) {
    let mut insert = AuditLog::new(DbConn::pool(db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_admin_id(admin_id)
        .set_action(action)
        .set_table_name(table_name.to_string())
        .set_record_key(record_key.to_string());

    if let Some(old) = old_data {
        insert = insert.set_old_data(Some(old));
    }
    if let Some(new) = new_data {
        insert = insert.set_new_data(Some(new));
    }

    let _ = insert.save().await;
}
