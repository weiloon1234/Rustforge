use axum::middleware::from_fn_with_state;
use core_web::openapi::{aide::axum::routing::get_with, ApiRouter};

use crate::internal::api::state::AppApiState;

mod auth;
mod team;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/auth", auth::router(state.clone()))
        .merge(guarded_router(state))
}

fn guarded_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/health",
            get_with(user_health, |op| {
                op.summary("User health").tag("User system")
            }),
        )
        .nest("/team", team::router(state.clone()))
        .layer(from_fn_with_state(
            state,
            crate::internal::middleware::auth::require_user,
        ))
}

async fn user_health() -> &'static str {
    "ok"
}
