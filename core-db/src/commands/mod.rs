use std::path::PathBuf;
use tokio::fs;

pub mod cli;
pub mod seeder;

fn migrations_dir() -> PathBuf {
    std::env::var("APP_MIGRATIONS_DIR")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("migrations"))
}

/// Commands for Framework Migrations
pub mod migrations {
    use super::*;

    /// Generate (pump) framework migration files into ./migrations
    pub async fn pump() -> anyhow::Result<()> {
        let migrations_dir = migrations_dir();
        fs::create_dir_all(&migrations_dir).await?;

        // 1. Meta
        let meta_sql = r#"
CREATE TABLE IF NOT EXISTS meta (
    id BIGINT PRIMARY KEY,
    owner_type TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    field TEXT NOT NULL,
    value JSONB NOT NULL DEFAULT '{}',
    UNIQUE (owner_type, owner_id, field)
);
CREATE INDEX IF NOT EXISTS idx_meta_owner ON meta(owner_type, owner_id);
"#;
        let meta_path = migrations_dir.join("0000000000001_meta.sql");
        fs::write(&meta_path, meta_sql).await?;
        println!("Created/Updated: {}", meta_path.display());

        // 2. Attachments
        let att_sql = r#"
CREATE TABLE IF NOT EXISTS attachments (
    id UUID PRIMARY KEY,
    owner_type TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    field TEXT NOT NULL,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    width INT,
    height INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_attachments_owner ON attachments(owner_type, owner_id);
"#;
        let att_path = migrations_dir.join("0000000000002_attachments.sql");
        fs::write(&att_path, att_sql).await?;
        println!("Created/Updated: {}", att_path.display());

        // 3. Localized
        let loc_sql = r#"
CREATE TABLE IF NOT EXISTS localized (
    id BIGINT PRIMARY KEY,
    owner_type TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    field TEXT NOT NULL,
    locale TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE (owner_type, owner_id, field, locale)
);
CREATE INDEX IF NOT EXISTS idx_localized_owner ON localized(owner_type, owner_id);
"#;
        let loc_path = migrations_dir.join("0000000000003_localized.sql");
        fs::write(&loc_path, loc_sql).await?;
        println!("Created/Updated: {}", loc_path.display());

        // 4. Personal Access Tokens
        let pat_sql = r#"
CREATE TABLE IF NOT EXISTS personal_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tokenable_type TEXT NOT NULL,
    tokenable_id TEXT NOT NULL,
    name TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    token_kind TEXT NOT NULL CHECK (token_kind IN ('access', 'refresh')),
    family_id UUID NOT NULL,
    parent_token_id UUID,
    abilities JSONB,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pat_tokenable_kind ON personal_access_tokens(tokenable_type, tokenable_id, token_kind);
CREATE INDEX IF NOT EXISTS idx_pat_token ON personal_access_tokens(token);
CREATE INDEX IF NOT EXISTS idx_pat_family_id ON personal_access_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_pat_parent_token_id ON personal_access_tokens(parent_token_id);
CREATE INDEX IF NOT EXISTS idx_pat_active_refresh ON personal_access_tokens(tokenable_type, tokenable_id, token_kind, revoked_at, expires_at);
"#;
        let pat_path = migrations_dir.join("0000000000004_personal_access_tokens.sql");
        fs::write(&pat_path, pat_sql).await?;
        println!("Created/Updated: {}", pat_path.display());

        // 5. Failed Jobs
        let failed_sql = r#"
CREATE TABLE IF NOT EXISTS failed_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_name TEXT NOT NULL,
    queue TEXT NOT NULL,
    payload JSONB NOT NULL,
    error TEXT NOT NULL,
    attempts INT NOT NULL,
    group_id TEXT,
    failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_failed_jobs_failed_at ON failed_jobs(failed_at);
CREATE INDEX IF NOT EXISTS idx_failed_jobs_group_id ON failed_jobs(group_id);
"#;
        let failed_path = migrations_dir.join("0000000000005_failed_jobs.sql");
        fs::write(&failed_path, failed_sql).await?;
        println!("Created/Updated: {}", failed_path.display());

        // 6. Outbox Jobs
        let outbox_sql = r#"
CREATE TABLE IF NOT EXISTS outbox_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_outbox_jobs_created_at ON outbox_jobs(created_at);
"#;
        let outbox_path = migrations_dir.join("0000000000006_outbox_jobs.sql");
        fs::write(&outbox_path, outbox_sql).await?;
        println!("Created/Updated: {}", outbox_path.display());

        // 7. Http Logs
        let logs_sql = r#"
CREATE TABLE IF NOT EXISTS webhook_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_url TEXT NOT NULL,
    request_method TEXT NOT NULL,
    request_headers JSONB,
    request_body TEXT,
    response_status INT,
    response_body TEXT,
    duration_ms INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_webhook_logs_created_at ON webhook_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_webhook_logs_url ON webhook_logs(request_url);
CREATE INDEX IF NOT EXISTS idx_webhook_logs_method ON webhook_logs(request_method);

CREATE TABLE IF NOT EXISTS http_client_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_url TEXT NOT NULL,
    request_method TEXT NOT NULL,
    request_headers JSONB,
    request_body TEXT,
    response_status INT,
    response_headers JSONB,
    response_body TEXT,
    duration_ms INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_http_client_logs_created_at ON http_client_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_http_client_logs_url ON http_client_logs(request_url);
CREATE INDEX IF NOT EXISTS idx_http_client_logs_method ON http_client_logs(request_method);
CREATE INDEX IF NOT EXISTS idx_http_client_logs_status ON http_client_logs(response_status);
"#;
        let logs_path = migrations_dir.join("0000000000007_http_logs.sql");
        fs::write(&logs_path, logs_sql).await?;
        println!("Created/Updated: {}", logs_path.display());

        // 8. Countries (framework reference data)
        let countries_sql = r#"
CREATE TABLE IF NOT EXISTS countries (
    iso2 TEXT PRIMARY KEY,
    iso3 TEXT NOT NULL UNIQUE,
    iso_numeric TEXT,
    name TEXT NOT NULL,
    official_name TEXT,
    capital TEXT,
    capitals TEXT[] NOT NULL DEFAULT '{}',
    region TEXT,
    subregion TEXT,
    currencies JSONB NOT NULL DEFAULT '[]',
    primary_currency_code TEXT,
    calling_code TEXT,
    calling_root TEXT,
    calling_suffixes TEXT[] NOT NULL DEFAULT '{}',
    tlds TEXT[] NOT NULL DEFAULT '{}',
    timezones TEXT[] NOT NULL DEFAULT '{}',
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    independent BOOLEAN,
    status TEXT NOT NULL DEFAULT 'disabled',
    assignment_status TEXT,
    un_member BOOLEAN,
    flag_emoji TEXT,
    is_default SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (iso2 ~ '^[A-Z]{2}$'),
    CHECK (iso3 ~ '^[A-Z]{3}$'),
    CHECK (iso_numeric IS NULL OR iso_numeric ~ '^[0-9]{3}$'),
    CHECK (status IN ('enabled', 'disabled'))
);
-- PK already guarantees unique + indexed lookup on iso2.
CREATE INDEX IF NOT EXISTS idx_countries_name ON countries(name);
CREATE INDEX IF NOT EXISTS idx_countries_status ON countries(status);
CREATE INDEX IF NOT EXISTS idx_countries_region ON countries(region);
CREATE INDEX IF NOT EXISTS idx_countries_primary_currency_code ON countries(primary_currency_code);
CREATE INDEX IF NOT EXISTS idx_countries_currencies_gin ON countries USING GIN (currencies);
CREATE INDEX IF NOT EXISTS idx_countries_is_default ON countries(is_default);
"#;
        let countries_path = migrations_dir.join("0000000000008_countries.sql");
        fs::write(&countries_path, countries_sql).await?;
        println!("Created/Updated: {}", countries_path.display());

        // 9. SQL Profiler
        let profiler_sql = r#"
CREATE TABLE IF NOT EXISTS sql_profiler_requests (
    id UUID PRIMARY KEY,
    request_method TEXT NOT NULL,
    request_path TEXT NOT NULL,
    total_queries INT NOT NULL DEFAULT 0,
    total_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sql_profiler_requests_created_at ON sql_profiler_requests(created_at);
CREATE INDEX IF NOT EXISTS idx_sql_profiler_requests_path ON sql_profiler_requests(request_path);

CREATE TABLE IF NOT EXISTS sql_profiler_queries (
    id BIGINT PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES sql_profiler_requests(id) ON DELETE CASCADE,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    sql TEXT NOT NULL,
    binds TEXT NOT NULL DEFAULT '',
    duration_us BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sql_profiler_queries_request_id ON sql_profiler_queries(request_id);
CREATE INDEX IF NOT EXISTS idx_sql_profiler_queries_table ON sql_profiler_queries(table_name);
CREATE INDEX IF NOT EXISTS idx_sql_profiler_queries_created_at ON sql_profiler_queries(created_at);
"#;
        let profiler_path = migrations_dir.join("0000000000009_sql_profiler.sql");
        fs::write(&profiler_path, profiler_sql).await?;
        println!("Created/Updated: {}", profiler_path.display());

        Ok(())
    }
}

/// Programmatic migration runner (no sqlx-cli dependency)
pub mod migrate_runner {
    use sqlx::postgres::PgPool;
    use std::path::Path;

    async fn connect() -> anyhow::Result<PgPool> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set for migrations"))?;
        Ok(PgPool::connect(&url).await?)
    }

    pub async fn run(migrations_dir: &Path) -> anyhow::Result<()> {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(migrations_dir).await?;
        migrator.run(&pool).await?;
        println!("Migrations applied successfully.");
        Ok(())
    }

    pub async fn revert(migrations_dir: &Path) -> anyhow::Result<()> {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(migrations_dir).await?;
        migrator.undo(&pool, 0).await?;
        println!("Last migration reverted.");
        Ok(())
    }

    pub async fn info(migrations_dir: &Path) -> anyhow::Result<()> {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(migrations_dir).await?;

        // Get applied migrations from DB
        let applied: std::collections::HashMap<i64, (bool, String)> = sqlx::query_as::<_, (i64, String, String)>(
            "SELECT version, description, checksum FROM _sqlx_migrations ORDER BY version"
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(v, d, c)| (v, (true, format!("{d} [{c}]"))))
        .collect();

        println!("{:<14} {:<50} {}", "Version", "Description", "Status");
        println!("{}", "-".repeat(80));

        for migration in migrator.iter() {
            let status = if applied.contains_key(&migration.version) {
                "applied"
            } else {
                "pending"
            };
            println!(
                "{:<14} {:<50} {}",
                migration.version,
                migration.description,
                status,
            );
        }
        Ok(())
    }

    pub async fn add(name: &str, migrations_dir: &Path) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(migrations_dir).await?;

        let now = time::OffsetDateTime::now_utc();
        let timestamp = now
            .format(&time::format_description::parse("[year][month][day][hour][minute][second]")?)
            .map_err(|e| anyhow::anyhow!("Failed to format timestamp: {e}"))?;
        let filename = format!("{}_{}.sql", timestamp, name);
        let path = migrations_dir.join(&filename);
        tokio::fs::write(&path, "-- Add migration SQL here\n").await?;
        println!("Created migration: {}", path.display());
        Ok(())
    }
}
