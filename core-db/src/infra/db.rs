use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use core_config::DbSettings;

pub async fn create_pool(settings: &DbSettings) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .acquire_timeout(settings.connect_timeout)
        .connect(&settings.url)
        .await
        .with_context(|| "Failed to connect to Postgres (DATABASE_URL)")?;

    Ok(pool)
}
