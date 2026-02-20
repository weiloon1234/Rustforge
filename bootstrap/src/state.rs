use crate::boot::BootContext;
use axum::extract::FromRef;
use core_db::infra::cache::Cache;
use core_db::infra::storage::Storage;
use sqlx::PgPool;
use std::sync::Arc;

/// A standard application state provided by the framework.
/// Projects can use this directly if they don't need custom fields.
#[derive(Clone)]
pub struct FrameworkState {
    pub db: PgPool,
    pub redis: Cache,
    pub storage: Arc<dyn Storage>,
    pub queue: core_jobs::queue::RedisQueue,

    // Configs often needed in handlers
    // Configs often needed in handlers
    pub settings: Arc<core_config::Settings>,
}

impl From<BootContext> for FrameworkState {
    fn from(ctx: BootContext) -> Self {
        Self {
            db: ctx.db,
            redis: ctx.redis,
            storage: ctx.storage,
            queue: ctx.queue,
            settings: ctx.settings,
        }
    }
}

// Implement FromRef so handlers can extract individual components easily
impl FromRef<FrameworkState> for PgPool {
    fn from_ref(state: &FrameworkState) -> Self {
        state.db.clone()
    }
}

impl FromRef<FrameworkState> for Cache {
    fn from_ref(state: &FrameworkState) -> Self {
        state.redis.clone()
    }
}

impl FromRef<FrameworkState> for core_jobs::queue::RedisQueue {
    fn from_ref(state: &FrameworkState) -> Self {
        state.queue.clone()
    }
}

impl FromRef<FrameworkState> for Arc<core_config::Settings> {
    fn from_ref(state: &FrameworkState) -> Self {
        state.settings.clone()
    }
}
