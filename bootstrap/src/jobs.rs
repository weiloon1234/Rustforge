use crate::boot::{init_app, BootContext};
use anyhow::Result;
use tracing::info;

/// Starts the worker with an existing boot context.
/// Use this when embedding the worker in another app (like api-server).
pub async fn start_with_context<F, S>(
    ctx: BootContext,
    register_jobs: F,
    register_schedule: Option<S>,
) -> Result<()>
where
    F: FnOnce(&mut core_jobs::worker::Worker),
    S: FnOnce(&mut core_jobs::cron::Scheduler),
{
    info!("Starting Worker...");

    // 2. Initialize Worker with Context
    let mut extensions = axum::http::Extensions::new();
    extensions.insert(ctx.mailer.clone());
    extensions.insert(ctx.storage.clone()); // Added explicit storage injection

    let job_ctx = core_jobs::JobContext {
        db: ctx.db.clone(),
        redis: ctx.redis.clone(),
        settings: ctx.settings.clone(),
        extensions,
    };

    // Use from_settings to ensure worker config (concurrency, sweeper) is loaded
    let mut worker = core_jobs::worker::Worker::from_settings(job_ctx).await?;

    // 3. Register Framework Jobs
    // Auto-register HTTP log cleanup if needed (or always available)
    worker.register::<core_http_log::jobs::CleanupHttpLogs>();

    // 4. Register Jobs (Project Level)
    register_jobs(&mut worker);

    // 4. Register & Spawn Scheduler (if provided)
    // 4. Register & Spawn Scheduler
    let queue_prefix = ctx
        .settings
        .redis
        .prefix
        .as_ref()
        .map(|prefix| format!("{prefix}:queue"))
        .unwrap_or_else(|| "queue".to_string());
    let queue =
        core_jobs::queue::RedisQueue::new_with_prefix(&ctx.settings.redis.url, &queue_prefix)?;
    let mut scheduler = core_jobs::cron::Scheduler::new(queue);

    // Framework Schedules
    if ctx.settings.http_log.retention_days > 0 {
        // Run daily at midnight
        scheduler.cron::<core_http_log::jobs::CleanupHttpLogs>("0 0 0 * * *");
    }

    // App Schedules
    if let Some(reg) = register_schedule {
        reg(&mut scheduler);
    }

    info!("Spawning Cron Scheduler...");
    tokio::spawn(async move {
        if let Err(e) = scheduler.run().await {
            tracing::error!("Cron Scheduler crashed: {}", e);
        }
    });

    // 5. Run Worker
    worker.run().await?;

    Ok(())
}

/// Starts the worker.
///
/// # Arguments
/// * `register_jobs` - A function that registers jobs on the worker.
/// * `register_schedule` - Optional function to register cron schedules.
pub async fn start_worker<F, S>(register_jobs: F, register_schedule: Option<S>) -> Result<()>
where
    F: FnOnce(&mut core_jobs::worker::Worker),
    S: FnOnce(&mut core_jobs::cron::Scheduler),
{
    // 1. Common Boot (includes DB/Redis connect)
    let (ctx, _guard) = init_app().await?;
    start_with_context(ctx, register_jobs, register_schedule).await
}
