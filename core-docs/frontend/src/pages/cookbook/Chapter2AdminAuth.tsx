import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter2AdminAuth() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 2B: Admin Auth (PAT Scopes + Refresh Rotation)
                </h1>
                <p className="text-xl text-gray-500">
                    Build admin login/refresh/logout/me with PAT-only scopes and web/mobile token
                    transport.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Read <a href="#/cookbook-chapter-2-validation-dto">Chapter 2A</a> first for
                    DTO organization, wrapper types, and the PATCH async-uniqueness pattern. This
                    chapter focuses on auth/session flow and admin auth wiring.
                </p>

                <h2>Step 0: Objective</h2>
                <ul>
                    <li>
                        Runtime permission source is only{' '}
                        <code>personal_access_tokens.abilities</code>.
                    </li>
                    <li>
                        Access token: short-lived bearer token. Refresh token: rotated one-time
                        token.
                    </li>
                    <li>
                        Web: refresh token in HttpOnly cookie. Mobile: refresh token in response
                        body.
                    </li>
                </ul>

                <h2>Step 1: API DTOs</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/admin/auth.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::auth::AuthClientType;
use core_web::contracts::rustforge_contract;
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLoginInput {
    #[rf(length(min = 3, max = 64))]
    #[rf(alpha_dash)]
    #[rf(openapi(hint = "Use a project-level UsernameString wrapper type for reusable username rules."))]
    pub username: String,

    #[rf(length(min = 8, max = 128))]
    pub password: String,

    pub client_type: AuthClientType,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminRefreshInput {
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminProfileUpdateInput {
    #[rf(length(min = 1, max = 120))]
    #[rf(required_trimmed)]
    pub name: String,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    pub current_password: String,
    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,
    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminAuthOutput {
    pub token_type: String,
    pub access_token: String,
    #[schemars(with = "Option<String>")]
    pub access_expires_at: Option<time::OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminMeOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub scopes: Vec<String>,
}`}</code>
                </pre>
                <p className="text-sm text-gray-600">
                    Use raw <code>#[validate(...)]</code> / <code>#[schemars(...)]</code> only for
                    explicit overrides or unsupported rule shapes (mainly schema-level validators
                    and async DB checks).
                </p>

                <h2>Step 2: Workflow Scope Grant</h2>
                <h3>
                    File: <code>app/src/internal/workflows/admin_auth.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::auth::{self, IssuedTokenPair, TokenScopeGrant};
use generated::{guards::AdminGuard, models::{AdminType, AdminView}, permissions::Permission};

pub fn resolve_scope_grant(admin: &AdminView) -> TokenScopeGrant {
    match admin.admin_type {
        AdminType::Developer | AdminType::SuperAdmin => TokenScopeGrant::Wildcard,
        AdminType::Admin => {
            let mut explicit = Vec::new();
            if let Some(items) = admin.abilities.as_array() {
                for item in items {
                    if let Some(raw) = item.as_str() {
                        if let Some(permission) = Permission::from_str(raw.trim()) {
                            explicit.push(permission.as_str().to_string());
                        }
                    }
                }
            }
            explicit.sort();
            explicit.dedup();
            if explicit.is_empty() {
                TokenScopeGrant::AuthOnly
            } else {
                TokenScopeGrant::Explicit(explicit)
            }
        }
    }
}

// login(...) -> issue_guard_session::<AdminGuard>(..., resolve_scope_grant(admin))
// refresh(...) -> refresh_guard_session::<AdminGuard>(...)
// revoke_session(...) -> revoke_session_by_refresh_token::<AdminGuard>(...)`}</code>
                </pre>
                <p>
                    This is the project override point. Replace <code>AuthOnly</code> with
                    <code>Explicit(Vec&lt;Permission&gt;)</code> when you need finer scopes.
                </p>

                <h2>Step 3: Admin Auth Routes</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/admin/auth.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub fn router(state: AppApiState) -> ApiRouter {
    let protected = ApiRouter::new()
        .api_route("/me", get(me))
        .api_route("/logout", post(logout))
        .api_route("/profile_update", patch(profile_update))
        .api_route("/password_update", patch(password_update))
        .layer(from_fn_with_state(
            state.clone(),
            crate::internal::middleware::auth::require_admin,
        ));

    ApiRouter::new()
        .api_route("/login", post(login))
        .api_route("/refresh", post(refresh))
        .merge(protected)
}

// Web refresh token source:
//   core_web::auth::extract_refresh_token_for_client(..., AuthClientType::Web, None)
// Mobile refresh token source:
//   body.refresh_token`}</code>
                </pre>

                <h2>Step 4: Portal Wiring</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user_router())
        .nest("/admin", admin_router(state))
}

fn admin_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/auth", admin_auth::router(state.clone()))
        .merge(admin_guarded_router(state))
}`}</code>
                </pre>

                <h2>Step 5: Permission Checks on Business Routes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`core_web::openapi::with_permission_check_post(
    create,
    generated::guards::AdminGuard,
    core_web::authz::PermissionMode::Any,
    [generated::permissions::Permission::ArticleManage],
);`}</code>
                </pre>

                <h2>Step 5A: Admin CRUD DTO Async Uniqueness</h2>
                <p>
                    The scaffolded admin CRUD module uses <code>rf(async_unique)</code> for
                    username uniqueness and the <code>__target_id</code> pattern for PATCH routes.
                    See{' '}
                    <a href="#/cookbook-chapter-2-validation-dto">Chapter 2A Steps 4-5</a> for the
                    full DTO definitions, handler wiring, and the{' '}
                    <code>AsyncContractJson</code> vs <code>ContractJson + validate_async</code>{' '}
                    decision rule.
                </p>

                <h2>Step 6: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check --workspace
./console migrate pump
./console migrate run
./console route list

# login
curl -sS -X POST http://127.0.0.1:3000/api/v1/admin/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"developer","password":"password123","client_type":"mobile"}'`}</code>
                </pre>
            </div>
        </div>
    )
}
