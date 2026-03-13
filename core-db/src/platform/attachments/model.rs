#[rf_model(table = "attachments", soft_delete)]
pub struct Attachment {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub owner_type: String,
    pub owner_id: i64,
    pub field: String,
    pub path: String,
    pub content_type: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
