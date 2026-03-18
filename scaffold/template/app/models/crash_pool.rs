#[rf_model(table = "crash_pools")]
pub struct CrashPool {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub room_key: String,
    pub slug: String,
    pub bet_amount: rust_decimal::Decimal,
    pub fee_rate: rust_decimal::Decimal,
    pub balance: rust_decimal::Decimal,
    pub round_number: i64,
    pub sort_order: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
