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
    owner_type TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    field TEXT NOT NULL,
    value JSONB NOT NULL DEFAULT '{}',
    PRIMARY KEY (owner_type, owner_id, field)
);
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
    owner_type TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    field TEXT NOT NULL,
    locale TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (owner_type, owner_id, field, locale)
);
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
    abilities JSONB,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pat_tokenable ON personal_access_tokens(tokenable_type, tokenable_id);
CREATE INDEX IF NOT EXISTS idx_pat_token ON personal_access_tokens(token);
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

        // 8. Auth Subject Permissions
        let authz_sql = r#"
CREATE TABLE IF NOT EXISTS auth_subject_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guard TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    permission TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_auth_subject_permissions_guard_subject_permission
    ON auth_subject_permissions(guard, subject_id, permission);
CREATE INDEX IF NOT EXISTS idx_auth_subject_permissions_guard_subject
    ON auth_subject_permissions(guard, subject_id);
CREATE INDEX IF NOT EXISTS idx_auth_subject_permissions_permission
    ON auth_subject_permissions(permission);
"#;
        let authz_path = migrations_dir.join("0000000000008_auth_subject_permissions.sql");
        fs::write(&authz_path, authz_sql).await?;
        println!("Created/Updated: {}", authz_path.display());

        // 9. Countries (framework reference data)
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (char_length(iso2) = 2),
    CHECK (char_length(iso3) = 3),
    CHECK (iso_numeric IS NULL OR char_length(iso_numeric) = 3),
    CHECK (status IN ('enabled', 'disabled'))
);
CREATE INDEX IF NOT EXISTS idx_countries_name ON countries(name);
CREATE INDEX IF NOT EXISTS idx_countries_status ON countries(status);
CREATE INDEX IF NOT EXISTS idx_countries_region ON countries(region);
CREATE INDEX IF NOT EXISTS idx_countries_primary_currency_code ON countries(primary_currency_code);
CREATE INDEX IF NOT EXISTS idx_countries_currencies_gin ON countries USING GIN (currencies);
"#;
        let countries_path = migrations_dir.join("0000000000009_countries.sql");
        fs::write(&countries_path, countries_sql).await?;
        println!("Created/Updated: {}", countries_path.display());

        Ok(())
    }
}

/// Commands relating to SQLx migration tool
pub mod sqlx_tool {
    use std::process::Command;

    pub enum MigrateCommand {
        Run,
        Revert,
        Info,
        Add { name: String },
    }

    pub fn handle(cmd: MigrateCommand) -> anyhow::Result<()> {
        let migrations_dir = super::migrations_dir();
        let migrations_dir = migrations_dir.to_string_lossy().to_string();

        let status = match cmd {
            MigrateCommand::Run => Command::new("sqlx")
                .args(["migrate", "run", "--source", &migrations_dir])
                .status()?,
            MigrateCommand::Revert => Command::new("sqlx")
                .args(["migrate", "revert", "--source", &migrations_dir])
                .status()?,
            MigrateCommand::Info => Command::new("sqlx")
                .args(["migrate", "info", "--source", &migrations_dir])
                .status()?,
            MigrateCommand::Add { name } => Command::new("sqlx")
                .arg("migrate")
                .arg("add")
                .arg(name)
                .arg("--source")
                .arg(&migrations_dir)
                .status()?,
        };

        if !status.success() {
            anyhow::bail!("Migration command failed");
        }
        Ok(())
    }
}
