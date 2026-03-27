---
name: add-job
description: Add a background job, cron schedule, or outbox-dispatched task
---

# Add a Background Job

## When to use what

| Pattern | Use case | Persistence | Retries | Ordering |
|---------|----------|-------------|---------|----------|
| **Job queue** | Heavy work, retriable, durable | Redis + failed_jobs table | Yes (backoff) | Yes (group_id) |
| **Outbox (JobBuffer)** | Job must be atomic with DB transaction | Postgres → Redis | Yes | No |
| **Cron schedule** | Periodic tasks (cleanup, reports) | Redis dedup lock | Yes | No |
| **`tokio::spawn`** | Fire-and-forget, fast, non-critical | None | No | No |

**Rule:** When in doubt, use the job queue. Use `tokio::spawn` only for truly lightweight, non-critical side effects (realtime broadcasts, cache invalidation).

---

## Step 1: Create the job struct

Create `app/src/internal/jobs/{name}.rs`:

```rust
use async_trait::async_trait;
use core_jobs::{Job, JobContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyJob {
    pub record_id: i64,
}

#[async_trait]
impl Job for MyJob {
    const NAME: &'static str = "my_job";
    const QUEUE: &'static str = "default";  // or a custom queue name

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // ctx.db, ctx.redis, ctx.settings are available
        // Business logic here...
        Ok(())
    }

    fn max_retries(&self) -> u32 {
        5  // Default. Override for custom retry count
    }

    // Optional: called when all retries exhausted
    async fn failed(&self, _ctx: &JobContext, error: &str) -> anyhow::Result<()> {
        tracing::error!("Job {} failed permanently: {}", Self::NAME, error);
        Ok(())
    }
}
```

Conventions:
- `NAME` must be unique across all jobs
- Payload fields are serialized to JSON — keep minimal, store IDs and fetch data in `handle()`
- Use a dedicated `QUEUE` for isolation (e.g., `"image_generation"` for heavy processing)
- Return `Err(...)` to trigger retry with backoff (5s → 30s → 1m → 5m → 10m)

### Ordered/sequential jobs

Jobs with the same `group_id` are processed one at a time within that group:

```rust
fn group_id(&self) -> Option<String> {
    Some(format!("user:{}", self.user_id))
}
```

## Step 2: Register the job

In `app/src/internal/jobs/mod.rs`:

```rust
pub mod my_job;

pub fn register_jobs(worker: &mut Worker) {
    worker.register::<my_job::MyJob>();
}
```

## Step 3: Dispatch from a workflow

### Standard dispatch (via Redis)

```rust
// In workflow — dispatch to Redis queue directly
my_job::MyJob { record_id: 123 }.dispatch(&state.queue).await?;
```

### Transactional dispatch (via outbox)

When the job MUST be atomic with a database transaction:

```rust
use core_jobs::buffer::JobBuffer;

let scope = db.begin_scope().await?;
{
    // Domain logic in same transaction
    MyModel::create(scope.conn())
        .set(MyCol::NAME, "test")
        .save()
        .await?;

    // Buffer job — inserted to outbox_jobs table, NOT Redis yet
    let mut buffer = JobBuffer::new(scope.conn());
    buffer.push(my_job::MyJob { record_id: 123 }).await?;
}
scope.commit().await?;
// Outbox sweeper flushes to Redis within [worker].sweep_interval seconds
```

If the transaction commits, the job WILL be processed. If it rolls back, the job is discarded.

## Step 4 (optional): Add a cron schedule

For periodic jobs (daily cleanup, hourly reports). The job must implement `Default + Clone`.

```rust
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {
    // Daily at midnight
    scheduler
        .cron::<my_job::MyCleanupJob>("0 0 0 * * *")
        .without_overlapping(300);  // 5-min TTL lock prevents overlap

    // Every 5 minutes
    scheduler.cron::<my_job::MyPollJob>("0 */5 * * * *");
}
```

Cron expression format: `sec min hour day_of_month month day_of_week`

## Step 5: Wire module export

Add `pub mod my_job;` to `app/src/internal/jobs/mod.rs`.

## Step 6: Verify

```bash
cargo check
```

---

## tokio::spawn (fire-and-forget)

For lightweight, non-critical async work — no registration needed:

```rust
let realtime = state.realtime.clone();
tokio::spawn(async move {
    let _ = realtime.publish("admin", "notification", &payload).await;
});
```

No persistence, no retries. If the server restarts, the work is lost.

---

## Configuration

```toml
[worker]
enabled = true       # Enable job processing
concurrency = 10     # Concurrent worker threads
sweep_interval = 30  # Outbox sweeper interval (seconds)
```

## Notes

- `JobContext` provides: `db`, `redis`, `settings`, `extensions`
- Failed jobs → `failed_jobs` table after max retries
- Retry backoff default: 5s → 30s → 1m → 5m → 10m (override via `fn backoff()`)
- Management API at `/jobs` endpoint: list queues, peek jobs, retry failed
- Common mistake: forgetting to register the job — it will dispatch but never process
