use core_web::middleware::throttle::ThrottleLayer;
use core_web::openapi::ApiRouter;

use crate::internal::api::state::AppApiState;

mod admin;
mod user;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
        .layer(ThrottleLayer::new(state.rate_limit_per_sec, state.rate_limit_burst))
}
