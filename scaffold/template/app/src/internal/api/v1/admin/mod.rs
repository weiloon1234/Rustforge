use axum::middleware::from_fn_with_state;
use core_web::openapi::{aide::axum::routing::get_with, ApiRouter};

use crate::internal::api::{datatable, state::AppApiState};

mod account;
mod auth;
mod bank;
mod company_bank_account;
mod company_crypto_account;
mod content_page;
mod content_page_multipart;
mod country;
mod crypto_network;
mod deposit;
mod developer_logs;
mod notification;
mod receipt_upload;
mod hierarchy;
mod introducer_change;
mod tiptap_upload;
mod user;
mod user_credit;
mod withdrawal;

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
        .nest("/users", user::router(state.clone()))
        .nest("/users/hierarchy", hierarchy::router(state.clone()))
        .nest("/users/credits", user_credit::router(state.clone()))
        .nest("/introducer_changes", introducer_change::router(state.clone()))
        .nest("/countries", country::router(state.clone()))
        .nest("/content_page", content_page::router(state.clone()))
        .nest("/uploads", tiptap_upload::router(state.clone()))
        .nest("/banks", bank::router(state.clone()))
        .nest("/crypto_networks", crypto_network::router(state.clone()))
        .nest("/company_bank_accounts", company_bank_account::router(state.clone()))
        .nest("/company_crypto_accounts", company_crypto_account::router(state.clone()))
        .nest("/notifications", notification::router(state.clone()))
        .nest("/deposits", deposit::router(state.clone()))
        .nest("/withdrawals", withdrawal::router(state.clone()))
        .nest("/developer/logs", developer_logs::router(state.clone()))
        .merge(datatable::router(state.clone()))
        .layer(from_fn_with_state(
            state,
            crate::internal::middleware::auth::require_admin_with_audit,
        ))
}

async fn admin_health() -> &'static str {
    "ok"
}
