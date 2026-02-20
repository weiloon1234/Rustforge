use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AuthSubjectPermissionRow {
    pub id: Uuid,
    pub guard: String,
    pub subject_id: Uuid,
    pub permission: String,
}
