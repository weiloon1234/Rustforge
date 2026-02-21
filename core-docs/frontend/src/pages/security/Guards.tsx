export function Guards() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Guards &amp; Auth</h1>
                <p className="text-xl text-gray-500">
                    Guard-based authentication with PAT-only scopes and refresh rotation.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>1) Guard Configuration</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`# app/configs.toml
[auth]
default = "admin"

[auth.guards.admin]
provider = "admin"
ttl_min = 30
refresh_ttl_days = 30`}</code>
                </pre>
                <p>
                    Guard structs are generated into <code>generated/src/guards</code> by db-gen.
                </p>

                <h2>2) AuthState Contract</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`impl core_web::auth::AuthState for AppApiState {
    fn auth_db(&self) -> &sqlx::PgPool {
        &self.db
    }
}`}</code>
                </pre>

                <h2>3) Guard Middleware</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use axum::{extract::{Request, State}, middleware::Next, response::Response};
use core_web::error::AppError;
use generated::guards::AdminGuard;

pub async fn require_admin(
    state: State<AppApiState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    core_web::auth::require_auth::<AdminGuard, AppApiState>(state, request, next).await
}`}</code>
                </pre>

                <h2>4) PAT Scope Model</h2>
                <p>
                    Runtime permissions come from <code>personal_access_tokens.abilities</code> only.
                    Scopes are snapshot at issue/refresh time.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::auth::{issue_guard_session, TokenScopeGrant};

let session = issue_guard_session::<AdminGuard>(
    &db,
    &settings.auth,
    admin.id,
    "admin-session",
    TokenScopeGrant::AuthOnly,
).await?;`}</code>
                </pre>

                <h2>5) Refresh Rotation</h2>
                <p>
                    Refresh tokens are one-time-use. Reuse is rejected and session family can be revoked.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::auth::{
    extract_refresh_token_for_client,
    refresh_guard_session,
    AuthClientType,
};

let refresh = extract_refresh_token_for_client(
    &headers,
    "admin",
    AuthClientType::Web,
    None,
).ok_or_else(|| AppError::BadRequest("Missing refresh token".to_string()))?;

let session = refresh_guard_session::<AdminGuard>(
    &db,
    &settings.auth,
    &refresh,
    "admin-session",
).await?;`}</code>
                </pre>

                <h2>6) Typed Permission Checks</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Route-level (runtime + OpenAPI):
core_web::openapi::with_permission_check_post(
    create,
    generated::guards::AdminGuard,
    core_web::authz::PermissionMode::Any,
    [generated::permissions::Permission::ArticleManage],
);

// Handler/workflow fallback:
core_web::authz::ensure_permissions(
    &auth,
    core_web::authz::PermissionMode::Any,
    &[generated::permissions::Permission::ArticleManage],
)?;`}</code>
                </pre>
            </div>
        </div>
    )
}
