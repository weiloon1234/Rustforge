use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MetaRow {
    pub owner_type: String,
    pub owner_id: i64,
    pub field: String,
    pub value: serde_json::Value,
}
