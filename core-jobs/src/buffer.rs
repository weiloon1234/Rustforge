use crate::{queue::RedisQueue, Job, JobPayload};
use sqlx::{Postgres, Row, Transaction};

/// Transactional Job Buffer (Outbox Pattern).
/// Jobs are inserted into `outbox_jobs` table within the provided transaction.
pub struct JobBuffer<'a> {
    tx: &'a mut Transaction<'a, Postgres>,
}

impl<'a> JobBuffer<'a> {
    pub fn new(tx: &'a mut Transaction<'a, Postgres>) -> Self {
        Self { tx }
    }

    /// Push a job to the outbox (Postgres).
    pub async fn push<J: Job>(&mut self, job: J) -> anyhow::Result<()> {
        let payload = JobPayload {
            job: J::NAME.to_string(),
            data: serde_json::to_value(job)?,
            queue: J::QUEUE.to_string(),
            attempts: 0,
        };
        // Just serialize to Value directly for JSONB? Or string.
        // sqlx maps serde_json::Value to JSONB.
        let payload_json = serde_json::to_value(payload)?;

        // We use full queue name "queue:default" or just "default"?
        // RedisQueue uses `prefix:queue`.
        // Let's store just the suffix "default" or "scanners".
        // The flushing logic will reconstruct the full key.
        let queue_suffix = J::QUEUE;

        sqlx::query("INSERT INTO outbox_jobs (queue, payload) VALUES ($1, $2)")
            .bind(queue_suffix)
            .bind(payload_json)
            .execute(&mut **self.tx)
            .await?;

        Ok(())
    }
}

/// Helper to flush the outbox to Redis.
/// Should be called after transaction commit, or by a background cron.
pub struct OutboxFlusher;

impl OutboxFlusher {
    pub async fn flush(db: &sqlx::PgPool, queue: &RedisQueue) -> anyhow::Result<usize> {
        // 1. Lock a batch first. We only delete after Redis push succeeds.
        let mut tx = db.begin().await?;
        let rows = sqlx::query(
            "SELECT id, queue, payload
             FROM outbox_jobs
             ORDER BY created_at ASC
             LIMIT 100
             FOR UPDATE SKIP LOCKED",
        )
        .fetch_all(&mut *tx)
        .await?;

        if rows.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        let mut pipe = redis::pipe();
        // We need a connection *outside* the loop or pipe assumes it
        let mut conn = queue.client.get_multiplexed_async_connection().await?;

        let mut ids = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get("id");
            let q_suffix: String = row.get("queue");
            let payload: serde_json::Value = row.get("payload");

            // Reconstruct payload string for Redis
            let payload_str = serde_json::to_string(&payload)?;

            // Reconstruct full redis key
            let target_queue = format!("{}:{}", queue.prefix, q_suffix);

            pipe.rpush(target_queue, payload_str);
            count += 1;
            ids.push(id);
        }

        // 2. Push batch to Redis.
        let _: () = pipe.query_async(&mut conn).await?;

        // 3. Remove rows only after Redis push succeeded.
        sqlx::query("DELETE FROM outbox_jobs WHERE id = ANY($1)")
            .bind(ids)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(count)
    }
}
