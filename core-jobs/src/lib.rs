use async_trait::async_trait;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

pub mod api;
pub mod buffer;
// pub mod config;
pub mod cron;
pub mod queue;
pub mod runtime;
pub mod utils;
pub mod worker;

#[derive(Clone)]
pub struct JobContext {
    // We can expose DB connection factory or pool
    pub db: sqlx::PgPool,
    pub redis: core_db::infra::cache::Cache, // Or redis::Client
    pub settings: std::sync::Arc<core_config::Settings>,
    // Extensions for extra services (e.g. ChannelManager)
    // Extensions for extra services (e.g. ChannelManager)
    pub extensions: axum::http::Extensions,
}

fn default_queue_name() -> String {
    "default".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobPayload {
    pub job: String,
    pub data: serde_json::Value,
    #[serde(default = "default_queue_name")]
    pub queue: String,
    #[serde(default)]
    pub attempts: u32,
}

#[async_trait]
pub trait Job: Serialize + DeserializeOwned + Send + Sync + Debug + 'static {
    /// Unique name for the job (e.g. "SendWelcomeEmail").
    /// Used for serialization/deserialization registry.
    const NAME: &'static str;

    /// The Queue name to push to. Default "default".
    const QUEUE: &'static str = "default";

    /// Execute the job logic.
    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()>;

    /// Optional grouping ID. Jobs with same ID are processed serially.
    fn group_id(&self) -> Option<String> {
        None
    }

    /// Maximum number of retries. Default 5.
    fn max_retries(&self) -> u32 {
        5
    }

    /// Backoff strategy (delay in seconds) based on attempt number.
    /// Default: 5s, 30s, 1m, 5m, 10m.
    fn backoff(&self, attempt: u32) -> u64 {
        match attempt {
            0 => 5,
            1 => 30,
            2 => 60,
            3 => 300,
            _ => 600,
        }
    }

    /// Dispatch this job to the queue.
    async fn dispatch(&self, queue: &queue::RedisQueue) -> anyhow::Result<()> {
        queue.push(self).await
    }
}
