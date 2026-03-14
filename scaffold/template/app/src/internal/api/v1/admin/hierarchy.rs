use axum::extract::{Path, Query, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    error::AppError,
    openapi::{with_permission_check_get_with, ApiRouter},
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::hierarchy::{
        AdminDownlineNode, AdminDownlinesOutput, ResolveUserQuery, ResolvedUser,
    },
    internal::{
        api::state::AppApiState,
        workflows::{introducer_change as ic_workflow, user_team as team_workflow},
    },
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/resolve",
            with_permission_check_get_with(
                resolve_user,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserHierarchy.as_str()],
                |op| op.summary("Resolve user by username").tag("Admin User Hierarchy"),
            ),
        )
        .api_route(
            "/{id}/downlines",
            with_permission_check_get_with(
                downlines,
                AdminGuard,
                PermissionMode::Any,
                [Permission::UserHierarchy.as_str()],
                |op| {
                    op.summary("List user direct downlines")
                        .tag("Admin User Hierarchy")
                },
            ),
        )
        .with_state(state)
}

async fn resolve_user(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Query(query): Query<ResolveUserQuery>,
) -> Result<ApiResponse<ResolvedUser>, AppError> {
    let user = ic_workflow::resolve_user_by_username(&state, &query.username)
        .await?;
    Ok(ApiResponse::success(
        ResolvedUser {
            id: user.id.into(),
            username: user.username,
            name: user.name,
            introducer_user_id: user.introducer_user_id.map(|id| id.into()),
        },
        &t("User found"),
    ))
}

async fn downlines(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminDownlinesOutput>, AppError> {
    let user = crate::internal::workflows::user_manage::detail(&state, id)
        .await?;
    let rows = team_workflow::list_downlines(&state, id).await?;

    let downlines = rows
        .into_iter()
        .map(|r| AdminDownlineNode {
            id: r.id.into(),
            uuid: r.uuid,
            username: r.username,
            name: r.name,
            downline_count: r.downline_count,
        })
        .collect();

    Ok(ApiResponse::success(
        AdminDownlinesOutput {
            parent_id: user.id.into(),
            parent_username: user.username,
            parent_name: user.name,
            parent_introducer_user_id: user.introducer_user_id.map(|id| id.into()),
            downlines,
        },
        &t("Downlines loaded"),
    ))
}
