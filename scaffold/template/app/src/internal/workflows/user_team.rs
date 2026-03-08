use core_web::error::AppError;

use crate::internal::api::state::AppApiState;

#[derive(Debug, sqlx::FromRow)]
pub struct DownlineRow {
    pub id: i64,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    pub downline_count: i64,
}

pub async fn list_downlines(
    state: &AppApiState,
    parent_user_id: i64,
) -> Result<Vec<DownlineRow>, AppError> {
    let rows = sqlx::query_as::<_, DownlineRow>(
        "SELECT u.id, u.uuid, u.username, u.name,
                (SELECT COUNT(*) FROM users WHERE introducer_user_id = u.id) AS downline_count
         FROM users u
         WHERE u.introducer_user_id = $1
         ORDER BY u.created_at ASC",
    )
    .bind(parent_user_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::from)?;
    Ok(rows)
}
