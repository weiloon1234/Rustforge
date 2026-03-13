#[rf_model(table = "introducer_changes")]
pub struct IntroducerChange {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub user_id: i64,
    pub from_user_id: Option<i64>,
    pub to_user_id: i64,
    pub admin_id: i64,
    pub remark: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "user_id")]
    pub user: BelongsTo<User>,
    #[rf(foreign_key = "from_user_id")]
    pub from_user: BelongsTo<User>,
    #[rf(foreign_key = "to_user_id")]
    pub to_user: BelongsTo<User>,
    #[rf(foreign_key = "admin_id")]
    pub admin: BelongsTo<Admin>,
}
