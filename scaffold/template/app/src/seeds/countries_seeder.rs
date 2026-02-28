use async_trait::async_trait;
use core_db::{
    common::sql::DbConn,
    platform::countries::repo::CountryRepo,
    seeder::Seeder,
};

#[derive(Debug, Default)]
pub struct CountriesSeeder;

#[async_trait]
impl Seeder for CountriesSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        CountryRepo::new(DbConn::pool(db)).seed_builtin().await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "CountriesSeeder"
    }
}
