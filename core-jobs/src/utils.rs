use core_config::Settings;
use std::future::Future;
use std::sync::Arc;

/// Spawns the embedded worker if `RUN_WORKER` env var is true.
///
/// # Arguments
/// * `settings` - The global application settings (Arc for cheap cloning).
/// * `runner` - An async function that runs the worker logic (e.g., `jobs::run_worker`).
pub fn spawn_embedded_worker<F, Fut>(settings: &Arc<Settings>, runner: F)
where
    F: FnOnce(sqlx::PgPool, core_db::infra::cache::Cache) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    if settings.worker.enabled {
        tracing::info!("RUN_WORKER=true: Spawning background worker in this process");

        let worker_settings = settings.clone();
        tokio::spawn(async move {
            let db: sqlx::PgPool = core_db::infra::db::create_pool(&worker_settings.db)
                .await
                .expect("Failed to create worker DB pool");

            let redis: core_db::infra::cache::Cache =
                core_db::infra::cache::create_cache(&worker_settings.redis)
                    .await
                    .expect("Failed to create worker Redis cache");

            if let Err(e) = runner(db, redis).await {
                tracing::error!("Embedded worker crashed: {e}");
            }
        });
    }
}
