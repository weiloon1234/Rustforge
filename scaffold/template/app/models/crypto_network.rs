#[rf_db_enum(storage = "i16")]
pub enum CryptoNetworkStatus {
    Enabled = 1,
    Disabled = 2,
}

#[rf_model(table = "crypto_networks")]
pub struct CryptoNetwork {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub status: CryptoNetworkStatus,
    pub sort_order: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(kind = "image")]
    pub logo: Attachment,
}
