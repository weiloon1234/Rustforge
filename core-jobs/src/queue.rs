use crate::{Job, JobPayload};
use redis::AsyncCommands;
use serde::Serialize;

#[derive(Clone)]
pub struct RedisQueue {
    pub client: redis::Client, // Using raw redis client for BLPOP
    pub prefix: String,
}

impl RedisQueue {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        Self::new_with_prefix(url, "queue")
    }

    pub fn new_with_prefix(url: &str, prefix: impl AsRef<str>) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self {
            client,
            prefix: normalize_prefix(prefix.as_ref()),
        })
    }

    pub fn from_client(client: redis::Client) -> Self {
        Self::from_client_with_prefix(client, "queue")
    }

    pub fn from_client_with_prefix(client: redis::Client, prefix: impl AsRef<str>) -> Self {
        Self {
            client,
            prefix: normalize_prefix(prefix.as_ref()),
        }
    }

    pub async fn push<J: Job + Serialize>(&self, job: &J) -> anyhow::Result<()> {
        let payload = JobPayload {
            job: J::NAME.to_string(),
            data: serde_json::to_value(job)?,
            queue: J::QUEUE.to_string(),
            attempts: 0,
        };
        let payload_str = serde_json::to_string(&payload)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        if let Some(group) = job.group_id() {
            // Ordered Queue Logic
            let group_queue = format!("{}:{}:{}", self.prefix, J::QUEUE, group);
            let meta_queue = format!("{}:{}:meta", self.prefix, J::QUEUE);

            // 1. Push job to group content list
            let _: () = conn.rpush(&group_queue, payload_str).await?;

            // 2. Notify meta queue that this group has work
            // Note: We might spam meta queue with same group ID multiple times.
            // Worker must handle this (e.g., if group is empty, ignore).
            let _: () = conn.rpush(&meta_queue, group).await?;
        } else {
            // Standard Queue Logic
            let key = format!("{}:{}", self.prefix, J::QUEUE);
            let _: () = conn.rpush(&key, payload_str).await?;
        }

        Ok(())
    }
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        "queue".to_string()
    } else {
        trimmed.to_string()
    }
}
