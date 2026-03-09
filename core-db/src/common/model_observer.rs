//! Model lifecycle observer using task-local variables.
//!
//! Set a global observer per-request (e.g. in admin middleware) via `scope_observer()`.
//! Generated model code will call observer hooks automatically for models with `audit = true`.

use std::sync::Arc;
use tokio::task_local;

/// Model lifecycle event data passed to observer hooks.
#[derive(Debug, Clone)]
pub struct ModelEvent {
    /// The database table name (e.g. "users", "admin").
    pub table: &'static str,
    /// The primary key of the affected record.
    pub record_id: i64,
}

/// Observer trait for model lifecycle hooks.
///
/// All methods have default no-op implementations so consumers
/// only need to implement the hooks they care about.
#[async_trait::async_trait]
pub trait ModelObserver: Send + Sync {
    /// Called before an INSERT is executed.
    async fn on_creating(&self, _event: &ModelEvent, _new_data: &serde_json::Value) {}

    /// Called after a successful INSERT.
    async fn on_created(&self, _event: &ModelEvent, _new_data: &serde_json::Value) {}

    /// Called before an UPDATE is executed. `old_data` contains current row state.
    async fn on_updating(
        &self,
        _event: &ModelEvent,
        _old_data: &serde_json::Value,
    ) {
    }

    /// Called after a successful UPDATE with both old and new state.
    async fn on_updated(
        &self,
        _event: &ModelEvent,
        _old_data: &serde_json::Value,
        _new_data: &serde_json::Value,
    ) {
    }

    /// Called before a DELETE is executed. `old_data` contains current row state.
    async fn on_deleting(
        &self,
        _event: &ModelEvent,
        _old_data: &serde_json::Value,
    ) {
    }

    /// Called after a successful DELETE.
    async fn on_deleted(
        &self,
        _event: &ModelEvent,
        _old_data: &serde_json::Value,
    ) {
    }
}

task_local! {
    static CURRENT_OBSERVER: Arc<dyn ModelObserver>;
}

/// Try to get the current task-local observer, if one is set.
pub fn try_get_observer() -> Option<Arc<dyn ModelObserver>> {
    CURRENT_OBSERVER.try_with(|o| o.clone()).ok()
}

/// Run an async block with a model observer set in task-local context.
///
/// Typically called in admin middleware to enable audit logging for the request scope.
pub async fn scope_observer<F, Fut, T>(observer: Arc<dyn ModelObserver>, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    CURRENT_OBSERVER.scope(observer, f()).await
}
