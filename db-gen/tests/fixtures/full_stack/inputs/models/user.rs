#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub profile_id: i64,
    #[rf(foreign_key = "profile_id")]
    pub profile: BelongsTo<Profile>,
}
