use async_trait::async_trait;

/// Trait for database seeders.
#[async_trait]
pub trait Seeder: Send + Sync {
    /// Run the seeder.
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()>;

    /// Get the name of the seeder (used for filtering).
    fn name(&self) -> &str;

    /// Whether this seeder should run during a default `db:seed` (without args).
    /// Defaults to true.
    fn run_by_default(&self) -> bool {
        true
    }
}
