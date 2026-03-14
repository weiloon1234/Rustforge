use axum::extract::State;
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    error::AppError,
    openapi::{aide::axum::routing::get_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::UserGuard, models::UserModel};

use crate::{
    contracts::api::v1::user::team::{DownlineNode, DownlinesOutput, DownlinesQuery},
    internal::{api::state::AppApiState, workflows::user_team as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/downlines",
            get_with(downlines, |op| {
                op.summary("List direct downlines").tag("User Team")
            }),
        )
        .with_state(state)
}

async fn downlines(
    State(state): State<AppApiState>,
    auth: AuthUser<UserGuard>,
    axum::extract::Query(query): axum::extract::Query<DownlinesQuery>,
) -> Result<ApiResponse<DownlinesOutput>, AppError> {
    let parent_id: i64 = match query.parent_user_id {
        Some(id) => id.into(),
        None => auth.user.id,
    };

    let parent = UserModel::find(core_db::common::sql::DbConn::pool(&state.db), parent_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("User not found")))?;

    let rows = workflow::list_downlines(&state, parent_id).await?;

    let downlines = rows
        .into_iter()
        .map(|r| DownlineNode {
            id: r.id.into(),
            uuid: r.uuid,
            username: r.username,
            name: r.name,
            downline_count: r.downline_count,
        })
        .collect();

    Ok(ApiResponse::success(
        DownlinesOutput {
            parent_username: parent.username,
            parent_name: parent.name,
            downlines,
        },
        &t("Downlines loaded"),
    ))
}
