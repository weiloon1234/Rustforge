#[rf_db_enum(storage = "i16")]
pub enum CompanyCryptoAccountStatus {
    Enabled = 1,
    Disabled = 2,
}

#[rf_model(table = "company_crypto_accounts")]
pub struct CompanyCryptoAccount {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub crypto_network_id: i64,
    pub wallet_address: String,
    pub conversion_rate: rust_decimal::Decimal,
    pub status: CompanyCryptoAccountStatus,
    pub sort_order: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "crypto_network_id")]
    pub crypto_network: BelongsTo<CryptoNetwork>,
}
