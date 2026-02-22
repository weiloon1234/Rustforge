import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter2ValidationDto() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 2A: Request/Response DTO + Validation
                </h1>
                <p className="text-xl text-gray-500">
                    Standardize contract DTOs, reusable rule types, and OpenAPI-aligned validation
                    before building larger modules.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Objective</h2>
                <ul>
                    <li>
                        DTO single source of truth lives under <code>app/src/contracts/*</code>.
                    </li>
                    <li>
                        Default DTO style is <code>#[rustforge_contract]</code> +{' '}
                        <code>#[rf(...)]</code>.
                    </li>
                    <li>
                        Use wrapper types for project-specific reusable rules (for example username,
                        slug, merchant code).
                    </li>
                    <li>
                        Keep handlers thin: validation at boundary, workflow handles business logic.
                    </li>
                </ul>

                <h2>Step 1: Organize DTO + Validation Files</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-text">{`app/
├── src/
│   ├── contracts/
│   │   ├── api/v1/
│   │   │   ├── admin.rs
│   │   │   ├── admin_auth.rs
│   │   │   └── ...
│   │   ├── datatable/admin/
│   │   │   └── admin.rs
│   │   └── types/
│   │       └── username.rs
│   └── validation/
│       ├── mod.rs
│       └── username.rs`}</code>
                </pre>
                <p>
                    <code>contracts/*</code> is the API contract SSOT. <code>validation/*</code>{' '}
                    contains custom reusable rule functions used by wrapper types.
                </p>

                <h2>Step 2: Use Wrapper Type for Project Custom Rules</h2>
                <h3>
                    File: <code>app/src/validation/username.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use validator::ValidationError;

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value)?;
    core_web::rules::alpha_dash(value)?;

    if value != value.to_ascii_lowercase() {
        let mut err = ValidationError::new("lowercase_username");
        err.message = Some("Username must be lowercase".into());
        return Err(err);
    }

    Ok(())
}`}</code>
                </pre>
                <h3>
                    File: <code>app/src/contracts/types/username.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::contracts::rustforge_string_rule_type;

rustforge_string_rule_type! {
    /// Lowercase username used for admin auth and admin CRUD.
    pub struct UsernameString {
        #[validate(custom(function = "crate::validation::username::validate_username"))]
        #[rf(length(min = 3, max = 64))]
        #[rf(rule = "alpha_dash")]
        #[rf(openapi_description = "Lowercase username using letters, numbers, underscore (_), and hyphen (-).")]
    }
}`}</code>
                </pre>
                <p>
                    DTO fields now use <code>UsernameString</code> + <code>#[rf(nested)]</code>{' '}
                    instead of repeating the same custom validation and OpenAPI hints everywhere.
                </p>

                <h2>Step 3: Request + Response DTO Style</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/admin_auth.rs</code> (excerpt)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use crate::contracts::types::username::UsernameString;
use core_web::auth::AuthClientType;
use core_web::contracts::rustforge_contract;
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminLoginInput {
    #[rf(nested)]
    pub username: UsernameString,

    #[rf(length(min = 8, max = 128))]
    pub password: String,

    pub client_type: AuthClientType,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
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

                <h2>Step 4: Async DB Validation in DTOs (Create)</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/admin.rs</code> (create excerpt)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct CreateAdminInput {
    #[rf(nested)]
    #[rf(async_unique(table = "admin", column = "username"))]
    pub username: UsernameString,

    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,

    #[rf(length(min = 1, max = 120))]
    pub name: String,
}`}</code>
                </pre>
                <p>
                    Create handlers can use <code>AsyncContractJson&lt;T&gt;</code> directly when
                    all async validation inputs come from the JSON body.
                </p>

                <h2>Step 5: PATCH Async Unique Pattern (Path ID is SSOT)</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/admin.rs</code> (update excerpt)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct UpdateAdminInput {
    #[serde(skip, default)]
    __target_id: i64,

    #[serde(default)]
    #[rf(nested)]
    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "__target_id")
    ))]
    pub username: Option<UsernameString>,
}

impl UpdateAdminInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}`}</code>
                </pre>
                <h3>
                    File: <code>app/src/internal/api/v1/admin.rs</code> (handler excerpt)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    contracts::{AsyncContractJson, ContractJson},
    error::AppError,
    extract::{validation::transform_validation_errors, AsyncValidate},
    response::ApiResponse,
};

async fn create(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: AsyncContractJson<CreateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::create(&state, &auth, req.0).await?;
    Ok(ApiResponse::success(AdminOutput::from(admin), &t("Admin created")))
}

async fn update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    req: ContractJson<UpdateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let req = req.0.with_target_id(id);
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }

    let admin = workflow::update(&state, &auth, id, req).await?;
    Ok(ApiResponse::success(AdminOutput::from(admin), &t("Admin updated")))
}`}</code>
                </pre>
                <p>
                    This pattern keeps <code>Path(id)</code> as the source of truth and avoids
                    duplicating uniqueness queries in workflows.
                </p>

                <h2>Step 6: OpenAPI + TypeScript Alignment</h2>
                <ul>
                    <li>
                        <code>#[rf(...)]</code> drives runtime validation generation and OpenAPI
                        hints/extensions (including <code>x-rf-rules</code>).
                    </li>
                    <li>
                        Enums in DTO fields should use generated enum types (for example{' '}
                        <code>generated::models::AdminType</code>) so OpenAPI shows selectable
                        options.
                    </li>
                    <li>
                        TypeScript generation remains OpenAPI-first from <code>/openapi.json</code>.
                    </li>
                </ul>
                <p>
                    See <a href="#/requests">Requests</a> and{' '}
                    <a href="#/validation-rules">Validation Rules</a> for the full rule list and
                    `rf(...)` syntax coverage.
                </p>

                <h2>Step 7: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check --workspace
./console route list

# check request schema and x-rf-rules
curl -sS http://127.0.0.1:3000/openapi.json | jq '.paths' > /tmp/openapi-paths.json`}</code>
                </pre>

                <h2>Next</h2>
                <p>
                    Continue with <a href="#/cookbook-chapter-2-admin-auth">Chapter 2B</a> to wire
                    PAT scopes, refresh rotation, and admin auth routes using the DTO patterns from
                    this chapter.
                </p>
            </div>
        </div>
    )
}

