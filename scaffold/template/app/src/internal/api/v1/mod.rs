use core_web::openapi::ApiRouter;

use crate::internal::api::state::AppApiState;

mod admin;
mod user;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user::router())
        .nest("/admin", admin::router(state))
}
