export function Guards() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Guards &amp; Auth</h1>
                <p className="text-xl text-gray-500">
                    Guard-based authentication, PAT session lifecycle, and account hydration boundaries.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Auth SSOT</h2>
                <p>
                    Guard configuration lives in <code>app/configs.toml</code>. Generated guard types,
                    token issue helpers, and auth extractors should stay aligned with that config instead of
                    repeating guard names across the codebase.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`[auth]
default = "admin"

[auth.guards.admin]
provider = "admin"
ttl_min = 30
refresh_ttl_days = 30`}</code>
                </pre>

                <h2>Generated/runtime boundary</h2>
                <ul>
                    <li>
                        db-gen emits guard types into <code>generated/src/guards</code>.
                    </li>
                    <li>
                        <code>AppApiState</code> implements <code>core_web::auth::AuthState</code> so
                        auth helpers know which database pool to use.
                    </li>
                    <li>
                        Protected routes should rely on generated guard types and typed auth extractors such as
                        <code>AuthUser&lt;AdminGuard&gt;</code>.
                    </li>
                </ul>

                <h2>Issue and refresh sessions</h2>
                <p>
                    Runtime abilities are stored on <code>personal_access_tokens.abilities</code> and are snapshotted
                    when sessions are issued or refreshed. There is no request-time permission-table join.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::auth::{issue_guard_session, refresh_guard_session, TokenScopeGrant};

let issued = issue_guard_session::<AdminGuard>(
    &db,
    &settings.auth,
    admin.id,
    "admin-session",
    TokenScopeGrant::AuthOnly,
).await?;

let refreshed = refresh_guard_session::<AdminGuard>(
    &db,
    &settings.auth,
    &refresh_token,
    "admin-session",
).await?;`}</code>
                </pre>

                <h2>Protected route pattern</h2>
                <p>
                    Prefer permission-aware route helpers for business endpoints. They keep runtime auth and OpenAPI
                    metadata aligned in one declaration.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::{with_permission_check_get, ApiRouter},
};
use generated::{guards::AdminGuard, permissions::Permission};

pub fn router(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route(
            "/me",
            with_permission_check_get(me, AdminGuard, PermissionMode::Any, [Permission::AdminRead]),
        )
        .with_state(state)
}`}</code>
                </pre>

                <h2>Login vs account hydration</h2>
                <p>
                    Treat login and account hydration as two separate concerns:
                </p>
                <ul>
                    <li>
                        <strong>Login:</strong> issue/refresh session tokens and return the token-scoped abilities snapshot.
                    </li>
                    <li>
                        <strong><code>/me</code>:</strong> hydrate the canonical account payload used by the frontend
                        for locale, admin type, and UI permission state.
                    </li>
                </ul>
                <p>
                    The frontend should call <code>/me</code> immediately after login rather than assuming the login
                    payload is the full account model.
                </p>

                <h2>Typed permission checks</h2>
                <p>
                    Keep guard/auth concerns separate from permission matching, but expose typed permission helpers
                    on top of the authenticated account model where it improves DX.
                </p>
                <ul>
                    <li>
                        Backend: use <code>ensure_permissions</code>, route helpers, or app-facing permission methods on generated views.
                    </li>
                    <li>
                        Frontend: keep raw <code>string[]</code> scopes because wildcard matching works on the
                        stored scope strings, but gate UI through typed store helpers.
                    </li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/permissions">Permissions &amp; AuthZ</a> for matcher rules and delegation policy.
                    </li>
                    <li>
                        <a href="#/openapi">OpenAPI</a> for the guard/permission metadata emitted on routes.
                    </li>
                    <li>
                        <a href="#/cookbook/add-admin-auth-permission-gates">Add Admin Auth &amp; Permission Gates</a> for the starter recipe.
                    </li>
                </ul>
            </div>
        </div>
    )
}
