export function Routing() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Routing</h1>
                <p className="text-xl text-gray-500">Define API endpoints with Axum + Aide.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Use <a href="https://github.com/tokio-rs/axum">Axum</a> for runtime and{' '}
                    <a href="https://github.com/tamasfe/aide">Aide</a> for OpenAPI inference.
                    Starter routing begins in <code>app/src/internal/api/mod.rs</code> and
                    <code> app/src/internal/api/v1/mod.rs</code>.
                </p>

                <h3>ApiRouter Example</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_web::openapi::{aide::axum::routing::get, ApiRouter};

pub fn router(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .nest("/user", user_router())
        .nest("/admin", admin_router(state))
}

fn user_router() -> ApiRouter<AppApiState> {
    ApiRouter::new().api_route("/health", get(user_health))
}`}</code>
                </pre>

                <h3>State + Middleware</h3>
                <p>
                    Keep portal prefixes in one place and apply auth middleware at portal
                    boundaries (for example admin routes).
                </p>
            </div>
        </div>
    )
}
