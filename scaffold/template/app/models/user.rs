#[rf_db_enum(storage = "i16")]
pub enum UserBanStatus {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "users")]
pub struct User {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    #[rf(hashed)]
    pub password: String,
    pub country_iso2: Option<String>,
    pub contact_number: Option<String>,
    pub introducer_user_id: Option<i64>,
    pub ban: UserBanStatus,
    pub credit_1: rust_decimal::Decimal,
    pub credit_2: rust_decimal::Decimal,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "introducer_user_id")]
    pub introducer: BelongsTo<User>,
    #[rf(foreign_key = "introducer_user_id")]
    pub downlines: HasMany<User>,
}
