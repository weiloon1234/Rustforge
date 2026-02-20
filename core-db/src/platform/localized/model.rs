use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct LocalizedRow {
    pub owner_type: String,
    pub owner_id: i64,
    pub field: String,
    pub locale: String,
    pub value: String,
}
