export function Guards() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Guards &amp; Auth</h1>
                <p className="text-xl text-gray-500">
                    Guard-based authentication with DB-fresh subject permissions.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>1) Guard Configuration</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`# app/configs.toml
[auth]
default = "admin"

[auth.guards.admin]
provider = "admin"`}</code>
                </pre>
                <p>
                    Guard structs are generated into <code>generated/src/guards</code> by
                    db-gen.
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

                <h2>4) Immediate Permission Freshness</h2>
                <p>
                    Each authenticated request loads permissions from{' '}
                    <code>auth_subject_permissions</code>. PAT abilities are compatibility fallback
                    only when subject rows are empty.
                </p>

                <h2>5) Typed Permission Checks</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Prefer route-level one-call helper:
core_web::openapi::with_permission_check_post(
    create,
    generated::guards::AdminGuard,
    core_web::authz::PermissionMode::Any,
    [generated::permissions::Permission::ArticleManage],
);

// Manual fallback in handler/workflow:
core_web::authz::ensure_permissions(
    &auth,
    core_web::authz::PermissionMode::Any,
    &[generated::permissions::Permission::ArticleManage],
)?;`}</code>
                </pre>

                <h2>6) Admin Type Helpers (App Extension)</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use models::admin_ext::AdminViewExt;

if auth.user.is_developer() {
    // developer-only branch
}`}</code>
                </pre>

                <h2>7) Permission Storage Repo</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_db::{
    common::sql::DbConn,
    platform::auth_subject_permissions::repo::AuthSubjectPermissionRepo,
};

let repo = AuthSubjectPermissionRepo::new(DbConn::pool(&db));
repo.grant("admin", admin_id, Permission::ArticleManage.as_str()).await?;
repo.revoke("admin", admin_id, Permission::ArticleManage.as_str()).await?;
repo.replace("admin", admin_id, &vec!["*".to_string()]).await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
