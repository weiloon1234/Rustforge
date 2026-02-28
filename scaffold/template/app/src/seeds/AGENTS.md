# Seeds

Database seeders for initial/test data. Implement the `Seeder` trait.

```rust
use async_trait::async_trait;
use core_db::seeder::Seeder;

pub struct ArticleSeeder;

#[async_trait]
impl Seeder for ArticleSeeder {
    fn name(&self) -> &str { "ArticleSeeder" }

    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        // Insert seed data
        Ok(())
    }
}
```

Register in `seeds/mod.rs` and pass to `bootstrap::console::start_console`.

Run: `./console db seed`
