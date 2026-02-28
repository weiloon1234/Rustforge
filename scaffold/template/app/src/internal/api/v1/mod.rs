use axum::middleware::from_fn_with_state;
use core_web::openapi::{
    aide::axum::routing::get_with,
    ApiRouter,
};

use crate::internal::api::{datatable, state::AppApiState};

mod admin;
mod admin_auth;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user_router())
        .nest("/admin", admin_router(state))
}

fn user_router() -> ApiRouter {
    ApiRouter::new().api_route(
        "/health",
        get_with(user_health, |op| op.summary("User health").tag("User system")),
    )
}

fn admin_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/auth", admin_auth::router(state.clone()))
        .merge(admin_guarded_router(state))
}

fn admin_guarded_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/health",
            get_with(admin_health, |op| op.summary("Admin health").tag("Admin system")),
        )
        .nest("/admins", admin::router(state.clone()))
        .merge(datatable::router(state.clone()))
        .layer(from_fn_with_state(
            state,
            crate::internal::middleware::auth::require_admin,
        ))
}

async fn user_health() -> &'static str {
    "ok"
}

async fn admin_health() -> &'static str {
    "ok"
}
