# Background Jobs

Define job structs, register them in this module, and dispatch from workflows.

## Define a Job

```rust
use async_trait::async_trait;
use core_jobs::{Job, JobContext};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendWelcomeEmailJob {
    pub user_id: i64,
    pub email: String,
}

#[async_trait]
impl Job for SendWelcomeEmailJob {
    const NAME: &'static str = "SendWelcomeEmail";
    const QUEUE: &'static str = "emails";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // ctx.db, ctx.redis, ctx.settings available
        Ok(())
    }

    fn max_retries(&self) -> u32 { 3 }
}
```

## Register

In `jobs/mod.rs`:
```rust
pub fn register_jobs(worker: &mut Worker) {
    worker.register::<SendWelcomeEmailJob>();
}

pub fn register_schedules(scheduler: &mut Scheduler) {
    scheduler.cron::<DailyCleanupJob>("0 2 * * *");
}
```

## Dispatch

```rust
let job = SendWelcomeEmailJob { user_id: 1, email: "a@b.com".into() };
job.dispatch(&state.queue).await?;
```
