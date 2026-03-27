---
name: add-job
description: Add a background job for async processing
---

# Add a Background Job

Follow these steps to add a background job for asynchronous processing.

## When to use a job queue vs tokio::spawn

- **Use the job queue** (this guide) when you need: persistence across restarts, retries on failure, delayed execution, or audit trail of job runs. This is the default for most background work.
- **Use `tokio::spawn`** for fire-and-forget tasks that are cheap, non-critical, and do not need retries or persistence (e.g., sending a single log event, lightweight cache warming). If the task fails silently or is lost on restart, it should not matter.

When in doubt, use the job queue.

## Step 1: Create the job file

Create `app/src/internal/jobs/{name}.rs`:

```rust
use rustforge_prelude::prelude::*;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyJob {
    pub entity_id: i64,
    pub payload: String,
}

#[async_trait]
impl Job for MyJob {
    const NAME: &'static str = "MyJob";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        let db = &ctx.state.db;

        // Job logic here
        tracing::info!(entity_id = self.entity_id, "Processing MyJob");

        // Example: fetch record, perform work, update status
        // let record = MyDomainModel::find_by_id(self.entity_id).one(db).await?;
        // ...

        Ok(())
    }
}
```

Conventions:
- `NAME` must be a unique string identifier for the job.
- The struct fields are the job payload -- they are serialized to JSON when dispatched and deserialized when processed.
- Keep payload minimal: store IDs and fetch full data inside `handle()`.
- Use `tracing` for structured logging inside jobs.
- Return `Err(...)` to mark the job as failed (it may be retried depending on configuration).

## Step 2: Register the job

Update `app/src/internal/jobs/mod.rs`:

```rust
pub mod {name};

// In the register function:
pub fn register(worker: &mut Worker) {
    // ... existing registrations
    worker.register::<{name}::MyJob>();
}
```

The job must be registered so the worker knows how to deserialize and handle it.

## Step 3: Dispatch the job

Dispatch the job from a workflow, handler, or observer:

```rust
use crate::internal::jobs::{name}::MyJob;

// In a workflow or handler:
state.queue.dispatch(&MyJob {
    entity_id: record.id,
    payload: "some data".into(),
}).await?;
```

To dispatch with a delay:
```rust
state.queue.dispatch_delayed(
    &MyJob { entity_id: record.id, payload: "data".into() },
    chrono::Duration::minutes(5),
).await?;
```

## Step 4: Wire the module export

Ensure `pub mod {name};` is in `app/src/internal/jobs/mod.rs`.

## Step 5: Verify

```bash
cargo check
```

Common issues:
- Forgetting to register the job in the worker -- the job will be dispatched but never processed.
- Payload struct not deriving both `Serialize` and `Deserialize`.
- Using non-serializable types in the payload (use IDs instead of full structs).
