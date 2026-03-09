use async_trait::async_trait;
use core_db::{common::auth::hash::hash_password, seeder::Seeder};

#[derive(Debug, Default)]
pub struct UserBootstrapSeeder;

#[async_trait]
impl Seeder for UserBootstrapSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        if should_skip_in_env() {
            return Ok(());
        }

        upsert_user(
            db,
            &env_or("SEED_USER_ORIGIN_USERNAME", "origin"),
            &env_or("SEED_USER_ORIGIN_PASSWORD", "123456789000"),
            optional_env("SEED_USER_ORIGIN_NAME").as_deref(),
        )
        .await?;

        Ok(())
    }

    fn name(&self) -> &str {
        "UserBootstrapSeeder"
    }
}

fn should_skip_in_env() -> bool {
    let app_env = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".to_string())
        .trim()
        .to_ascii_lowercase();

    if app_env != "production" {
        return false;
    }

    let raw = env_or("SEED_USER_BOOTSTRAP_IN_PROD", "");
    !is_truthy(&raw)
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "y"
    )
}

fn env_or(key: &str, default: &str) -> String {
    if let Ok(value) = std::env::var(key) {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }
    default.to_string()
}

fn optional_env(key: &str) -> Option<String> {
    if let Ok(value) = std::env::var(key) {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

async fn upsert_user(
    db: &sqlx::PgPool,
    username: &str,
    password_plain: &str,
    name: Option<&str>,
) -> anyhow::Result<i64> {
    let password = hash_password(password_plain)?;
    let id_to_insert = core_db::common::sql::generate_snowflake_i64();
    let username = username.trim().to_ascii_lowercase();
    let uuid = nanoid::nanoid!(8);

    let id = sqlx::query_scalar::<_, i64>(
        "
        INSERT INTO users (id, uuid, username, password, name, ban)
        VALUES ($1, $2, $3, $4, $5, 0)
        ON CONFLICT (username) DO UPDATE
        SET
            password = EXCLUDED.password,
            name = EXCLUDED.name,
            updated_at = NOW()
        RETURNING id
        ",
    )
    .bind(id_to_insert)
    .bind(uuid)
    .bind(username)
    .bind(password)
    .bind(name)
    .fetch_one(db)
    .await?;

    Ok(id)
}
