use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PersonalAccessTokenRow {
    pub id: Uuid,
    pub tokenable_type: String,
    pub tokenable_id: String,
    pub name: String,
    pub token: String,
    pub abilities: Option<sqlx::types::Json<Vec<String>>>, // JSON List of strings
    pub last_used_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
