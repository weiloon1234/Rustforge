use crate::{queue::RedisQueue, Job, JobPayload};
use core_db::{
    common::sql::{DbConn, Op, OrderDir},
    generated::models::{OutboxJob, OutboxJobCol},
};

/// Transactional Job Buffer (Outbox Pattern).
/// Jobs are inserted into `outbox_jobs` table within the provided DbConn scope.
pub struct JobBuffer<'a> {
    db: DbConn<'a>,
}

impl<'a> JobBuffer<'a> {
    pub fn new(db: impl Into<DbConn<'a>>) -> Self {
        Self { db: db.into() }
    }

    /// Push a job to the outbox (Postgres).
    pub async fn push<J: Job>(&mut self, job: J) -> anyhow::Result<()> {
        let payload = JobPayload {
            job: J::NAME.to_string(),
            data: serde_json::to_value(job)?,
            queue: J::QUEUE.to_string(),
            attempts: 0,
        };
        let payload_json = serde_json::to_value(payload)?;

        OutboxJob::new(self.db.clone(), None)
            .insert()
            .set_queue(J::QUEUE.to_string())
            .set_payload(payload_json)
            .save()
            .await?;

        Ok(())
    }
}

/// Helper to flush the outbox to Redis.
/// Should be called after transaction commit, or by a background cron.
pub struct OutboxFlusher;

impl OutboxFlusher {
    pub async fn flush(db: &sqlx::PgPool, queue: &RedisQueue) -> anyhow::Result<usize> {
        let pool_conn = DbConn::pool(db);
        let scope = pool_conn.begin_scope().await?;

        let count = {
            let conn = scope.conn();
            let rows = OutboxJob::new(conn.clone(), None)
                .query()
                .order_by(OutboxJobCol::CreatedAt, OrderDir::Asc)
                .for_update_skip_locked()
                .limit(100)
                .get()
                .await?;

            if rows.is_empty() {
                0usize
            } else {
                let mut count = 0usize;
                let mut pipe = redis::pipe();
                let mut redis_conn = queue.client.get_multiplexed_async_connection().await?;
                let mut ids = Vec::with_capacity(rows.len());

                for row in rows {
                    let payload_str = serde_json::to_string(&row.payload)?;
                    let target_queue = format!("{}:{}", queue.prefix, row.queue);
                    pipe.rpush(target_queue, payload_str);
                    ids.push(row.id);
                    count += 1;
                }

                let _: () = pipe.query_async(&mut redis_conn).await?;

                for id in ids {
                    OutboxJob::new(conn.clone(), None)
                        .query()
                        .where_id(Op::Eq, id)
                        .delete()
                        .await?;
                }
                count
            }
        };

        scope.commit().await?;
        Ok(count)
    }
}
