export function Jobs() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Job Queue</h1>
                <p className="text-xl text-gray-500">
                    Durable queue jobs plus lightweight goroutine-style tasks.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Use the framework in two async modes:
                    <br />
                    1) <strong>Durable Redis jobs</strong> for reliable background work.
                    <br />
                    2) <strong>In-process runtime tasks</strong> for goroutine-style fan-out and channels.
                </p>

                <h3>1) Durable Jobs (Laravel-style queue)</h3>
                <p>Implement `core_jobs::Job` and set `NAME` + `QUEUE`.</p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_jobs::{Job, JobContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SendWelcomeEmail {
    pub user_id: Uuid,
}

#[async_trait::async_trait]
impl Job for SendWelcomeEmail {
    const NAME: &'static str = "SendWelcomeEmail";
    const QUEUE: &'static str = "mail";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // Worker logic (DB, mailer, external APIs...)
        Ok(())
    }
}`}</code>
                </pre>

                <h3>Dispatching</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// In API/App layer
SendWelcomeEmail { user_id }.dispatch(&ctx.queue).await?;`}</code>
                </pre>

                <h3>Transactional Dispatch (after-commit pattern)</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_jobs::buffer::JobBuffer;

let mut tx = ctx.db.begin().await?;

// 1) Domain writes
// sqlx::query(...).execute(&mut *tx).await?;

// 2) Queue writes in same transaction
let mut buffer = JobBuffer::new(&mut tx);
buffer.push(SendWelcomeEmail { user_id }).await?;

// 3) Commit once
tx.commit().await?;`}</code>
                </pre>
                <p>
                    Jobs buffered into <code>outbox_jobs</code> are flushed by the worker sweeper
                    (`WORKER_SWEEP_INTERVAL`). This gives an after-commit safety net even if the
                    process crashes between commit and dispatch.
                </p>

                <h3>Registering Jobs in Worker</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// app/src/internal/jobs/lib.rs
pub fn register_jobs(worker: &mut core_jobs::worker::Worker) {
    worker.register::<SendWelcomeEmail>();
}`}</code>
                </pre>

                <h3>2) Goroutine-style Local Tasks</h3>
                <p>
                    For in-process concurrency (no Redis durability), use runtime helpers from
                    <code>core_jobs::runtime</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_jobs::runtime;

let (tx, mut rx) = runtime::channel::<String>(32);

runtime::go(async move {
    let _ = tx.send("hello from task".to_string()).await;
});

while let Some(msg) = rx.recv().await {
    tracing::info!("received: {}", msg);
    break;
}`}</code>
                </pre>
            </div>
        </div>
    )
}
