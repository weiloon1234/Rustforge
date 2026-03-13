#[rf_db_enum(storage = "i16")]
pub enum BankStatus {
    Enabled = 1,
    Disabled = 2,
}

#[rf_model(table = "banks")]
pub struct Bank {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub country_iso2: String,
    pub name: String,
    pub code: Option<String>,
    pub status: BankStatus,
    pub sort_order: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(kind = "image")]
    pub logo: Attachment,
}
