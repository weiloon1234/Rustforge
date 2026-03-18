use async_trait::async_trait;
use core_db::seeder::Seeder;

#[derive(Debug, Default)]
pub struct CrashPoolsSeeder;

#[async_trait]
impl Seeder for CrashPoolsSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        // (room_key, slug, bet_amount, sort_order) — ordered small to big
        let pools = [
            ("$1", "normal", "1.00", 1),
            ("$5", "platinum", "5.00", 2),
            ("$10", "gold", "10.00", 3),
            ("$100", "supreme", "100.00", 4),
        ];

        for (room_key, slug, bet_amount, sort_order) in pools {
            let id = core_db::common::sql::generate_snowflake_i64();
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO crash_pools (id, room_key, slug, bet_amount, sort_order, balance)
                 VALUES ($1, $2, $3, $4::numeric, $5, 0)
                 ON CONFLICT (room_key) DO UPDATE
                 SET bet_amount = EXCLUDED.bet_amount,
                     slug = EXCLUDED.slug,
                     sort_order = EXCLUDED.sort_order,
                     balance = 0,
                     updated_at = NOW()
                 RETURNING id",
            )
            .bind(id)
            .bind(room_key)
            .bind(slug)
            .bind(bet_amount)
            .bind(sort_order)
            .fetch_one(db)
            .await?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "CrashPoolsSeeder"
    }
}
