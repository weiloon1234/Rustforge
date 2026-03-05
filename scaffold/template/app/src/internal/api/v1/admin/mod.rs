use axum::middleware::from_fn_with_state;
use core_web::openapi::{aide::axum::routing::get_with, ApiRouter};

use crate::internal::api::{datatable, state::AppApiState};

mod account;
mod auth;
mod content_page;
mod content_page_multipart;
mod country;
mod tiptap_upload;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/auth", auth::router(state.clone()))
        .merge(guarded_router(state))
}

fn guarded_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/health",
            get_with(admin_health, |op| {
                op.summary("Admin health").tag("Admin system")
            }),
        )
        .nest("/admins", account::router(state.clone()))
        .nest("/countries", country::router(state.clone()))
        .nest("/content_page", content_page::router(state.clone()))
        .nest("/uploads", tiptap_upload::router(state.clone()))
        .merge(datatable::router(state.clone()))
        .layer(from_fn_with_state(
            state,
            crate::internal::middleware::auth::require_admin,
        ))
}

async fn admin_health() -> &'static str {
    "ok"
}
