use core_db::common::model_observer::{ModelEvent, ModelObserver};
use core_db::common::sql::{generate_snowflake_i64, DbConn};
use generated::models::{AuditAction, AuditLog};
use serde::Serialize;

use crate::internal::api::state::AppApiState;

/// Observer that automatically writes audit logs for model lifecycle events.
/// Set via `scope_observer()` in admin middleware.
pub struct AuditObserver {
    pub db: sqlx::PgPool,
    pub admin_id: i64,
}

#[async_trait::async_trait]
impl ModelObserver for AuditObserver {
    async fn on_created(&self, event: &ModelEvent, new_data: &serde_json::Value) {
        write_log_raw(
            &self.db,
            self.admin_id,
            AuditAction::Create,
            event.table,
            event.record_id,
            None,
            Some(new_data.clone()),
        )
        .await;
    }

    async fn on_updated(
        &self,
        event: &ModelEvent,
        old_data: &serde_json::Value,
        new_data: &serde_json::Value,
    ) {
        let (old_diff, new_diff) = compute_diff(old_data, new_data);
        if old_diff.is_none() && new_diff.is_none() {
            return; // No actual changes
        }
        write_log_raw(
            &self.db,
            self.admin_id,
            AuditAction::Update,
            event.table,
            event.record_id,
            old_diff,
            new_diff,
        )
        .await;
    }

    async fn on_deleted(&self, event: &ModelEvent, old_data: &serde_json::Value) {
        write_log_raw(
            &self.db,
            self.admin_id,
            AuditAction::Delete,
            event.table,
            event.record_id,
            Some(old_data.clone()),
            None,
        )
        .await;
    }
}

/// Compute dirty diff between old and new JSON objects.
/// Returns (old_diff, new_diff) with only changed fields, or (None, None) if no changes.
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

/// Log a create action. Captures the full new record as `new_data`.
pub async fn log_create<T: Serialize>(
    state: &AppApiState,
    admin_id: i64,
    table_name: &str,
    record_id: i64,
    new_record: &T,
) {
    let new_data = serde_json::to_value(new_record).ok();
    write_log(state, admin_id, AuditAction::Create, table_name, record_id, None, new_data).await;
}

/// Log an update action. Computes dirty diff between old and new.
/// Only writes if there are actual changes.
pub async fn log_update<T: Serialize>(
    state: &AppApiState,
    admin_id: i64,
    table_name: &str,
    record_id: i64,
    old_record: &T,
    new_record: &T,
) {
    let old_json = serde_json::to_value(old_record).ok();
    let new_json = serde_json::to_value(new_record).ok();

    let (old_diff, new_diff) = match (&old_json, &new_json) {
        (Some(serde_json::Value::Object(old_map)), Some(serde_json::Value::Object(new_map))) => {
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
                return; // No actual changes — skip audit
            }
            (
                Some(serde_json::Value::Object(old_changes)),
                Some(serde_json::Value::Object(new_changes)),
            )
        }
        _ => (old_json, new_json),
    };

    write_log(state, admin_id, AuditAction::Update, table_name, record_id, old_diff, new_diff).await;
}

/// Log a delete action. Captures the full old record as `old_data`.
pub async fn log_delete<T: Serialize>(
    state: &AppApiState,
    admin_id: i64,
    table_name: &str,
    record_id: i64,
    old_record: &T,
) {
    let old_data = serde_json::to_value(old_record).ok();
    write_log(state, admin_id, AuditAction::Delete, table_name, record_id, old_data, None).await;
}

async fn write_log(
    state: &AppApiState,
    admin_id: i64,
    action: AuditAction,
    table_name: &str,
    record_id: i64,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
) {
    write_log_raw(&state.db, admin_id, action, table_name, record_id, old_data, new_data).await;
}

async fn write_log_raw(
    db: &sqlx::PgPool,
    admin_id: i64,
    action: AuditAction,
    table_name: &str,
    record_id: i64,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
) {
    let mut insert = AuditLog::new(DbConn::pool(db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_admin_id(admin_id)
        .set_action(action)
        .set_table_name(table_name.to_string())
        .set_record_id(record_id);

    if let Some(old) = old_data {
        insert = insert.set_old_data(Some(old));
    }
    if let Some(new) = new_data {
        insert = insert.set_new_data(Some(new));
    }

    // Fire-and-forget: audit failures should not break the main operation
    let _ = insert.save().await;
}
