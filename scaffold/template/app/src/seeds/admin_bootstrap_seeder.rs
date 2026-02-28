use async_trait::async_trait;
use core_db::{
    common::auth::hash::hash_password,
    seeder::Seeder,
};

#[derive(Debug, Default)]
pub struct AdminBootstrapSeeder;

#[async_trait]
impl Seeder for AdminBootstrapSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        if should_skip_in_env() {
            return Ok(());
        }

        upsert_admin(
            db,
            &env_or("SEED_ADMIN_DEVELOPER_USERNAME", "developer"),
            optional_env("SEED_ADMIN_DEVELOPER_EMAIL"),
            &env_or("SEED_ADMIN_DEVELOPER_PASSWORD", "password123"),
            &env_or("SEED_ADMIN_DEVELOPER_NAME", "Developer"),
            "developer",
        )
        .await?;

        upsert_admin(
            db,
            &env_or("SEED_ADMIN_SUPERADMIN_USERNAME", "superadmin"),
            optional_env("SEED_ADMIN_SUPERADMIN_EMAIL"),
            &env_or("SEED_ADMIN_SUPERADMIN_PASSWORD", "password123"),
            &env_or("SEED_ADMIN_SUPERADMIN_NAME", "Super Admin"),
            "superadmin",
        )
        .await?;

        Ok(())
    }

    fn name(&self) -> &str {
        "AdminBootstrapSeeder"
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

    let raw = env_or("SEED_ADMIN_BOOTSTRAP_IN_PROD", "");
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
            return Some(value.to_ascii_lowercase());
        }
    }
    None
}

async fn upsert_admin(
    db: &sqlx::PgPool,
    username: &str,
    email: Option<String>,
    password_plain: &str,
    name: &str,
    admin_type: &str,
) -> anyhow::Result<i64> {
    let password = hash_password(password_plain)?;
    let id_to_insert = core_db::common::sql::generate_snowflake_i64();
    let username = username.trim().to_ascii_lowercase();

    let id = sqlx::query_scalar::<_, i64>(
        "\n        INSERT INTO admin (id, username, email, password, name, admin_type, abilities)\n        VALUES ($1, $2, $3, $4, $5, $6, '[]'::jsonb)\n        ON CONFLICT (username) DO UPDATE\n        SET\n            email = EXCLUDED.email,\n            password = EXCLUDED.password,\n            name = EXCLUDED.name,\n            admin_type = EXCLUDED.admin_type,\n            updated_at = NOW()\n        RETURNING id\n        ",
    )
    .bind(id_to_insert)
    .bind(username)
    .bind(email)
    .bind(password)
    .bind(name)
    .bind(admin_type)
    .fetch_one(db)
    .await?;

    Ok(id)
}
