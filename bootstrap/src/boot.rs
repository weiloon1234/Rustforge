use anyhow::Result;
use core_config::Settings;
use sqlx::PgPool;
use tracing::info;

use std::sync::Arc;

#[derive(Clone)]
pub struct BootContext {
    pub settings: Arc<Settings>,
    pub db: PgPool,
    pub redis: core_db::infra::cache::Cache,
    pub storage: std::sync::Arc<dyn core_db::infra::storage::Storage>,
    pub queue: core_jobs::queue::RedisQueue,
    // pub channel_manager: std::sync::Arc<core_notify::ChannelManager>,
    pub mailer: std::sync::Arc<core_mailer::Mailer>,
}

pub async fn init_app() -> Result<(BootContext, core_web::logging::WorkerGuard)> {
    // 1. Load env
    dotenvy::dotenv().ok();

    // 2. Init Logger
    let log_guard = core_web::logging::init();

    // 3. Load Settings
    let settings = Settings::load()?;
    info!("Bootstrapping {} ({})", settings.app.name, settings.app.env);

    // 4. Init I18n (default locale + timezone + catalogs)
    core_i18n::init(&settings.i18n);

    // 5. Connect DB
    let db = core_db::infra::db::create_pool(&settings.db).await?;

    // 6. Connect Redis
    let redis = core_db::infra::cache::create_cache(&settings.redis).await?;

    // 7. Init Storage
    // 7. Init Storage
    let storage = core_db::infra::storage::create_storage(&settings.s3).await?;

    // 8. Init Queue
    // Using Queue from Redis URL
    let queue_prefix = settings
        .redis
        .prefix
        .as_ref()
        .map(|prefix| format!("{prefix}:queue"))
        .unwrap_or_else(|| "queue".to_string());
    let queue = core_jobs::queue::RedisQueue::new_with_prefix(&settings.redis.url, &queue_prefix)?;

    // 9. Init Mailer
    let mailer = core_mailer::Mailer::new(&settings.mail, Some(queue.clone()))?;

    Ok((
        BootContext {
            settings: Arc::new(settings),
            db,
            redis,
            storage,
            queue,
            mailer: Arc::new(mailer), // Add mailer to context
        },
        log_guard,
    ))
}
