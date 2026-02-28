use core_web::openapi::{aide::axum::routing::get_with, ApiRouter};

pub fn router() -> ApiRouter {
    ApiRouter::new().api_route(
        "/health",
        get_with(user_health, |op| {
            op.summary("User health").tag("User system")
        }),
    )
}

async fn user_health() -> &'static str {
    "ok"
}
