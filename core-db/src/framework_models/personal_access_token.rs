#[rf_db_enum(storage = "string")]
pub enum PersonalAccessTokenKind {
    Access,
    Refresh,
}

#[rf_model(table = "personal_access_tokens")]
pub struct PersonalAccessToken {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub tokenable_type: String,
    pub tokenable_id: String,
    pub name: String,
    pub token: String,
    pub token_kind: PersonalAccessTokenKind,
    pub family_id: uuid::Uuid,
    pub parent_token_id: Option<uuid::Uuid>,
    pub abilities: Option<serde_json::Value>,
    pub last_used_at: Option<time::OffsetDateTime>,
    pub expires_at: Option<time::OffsetDateTime>,
    pub revoked_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
