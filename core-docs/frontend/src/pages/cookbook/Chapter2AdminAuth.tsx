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
                    Chapter 2: Request/Response DTO + Validation (extends Chapter 1)
                </h1>
                <p className="text-xl text-gray-500">
                    Build a clean DTO layer with reusable validation utilities and OpenAPI-aligned
                    schema hints.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <p>
                    This chapter extends Chapter 1. Focus is DTO and validation architecture only
                    (not schema/migration/authz).
                </p>
                <ul>
                    <li>
                        DTO SSOT: <code>app/src/contracts/api/*</code>.
                    </li>
                    <li>
                        Reusable app validators: <code>app/src/contracts/validation/*</code>.
                    </li>
                    <li>
                        Framework validators: <code>core_web::rules</code>.
                    </li>
                </ul>

                <h2>Step 1: Create DTO + Validation Folders</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`app/src/contracts/lib.rs
app/src/contracts/api/mod.rs
app/src/contracts/api/v1/mod.rs
app/src/contracts/api/v1/article.rs
app/src/contracts/validation/mod.rs
app/src/contracts/validation/sync.rs
app/src/contracts/validation/db.rs`}</code>
                </pre>
                <p>
                    Keep handlers/workflows free from inline DTO struct definitions. Import from{' '}
                    <code>contracts</code> crate only.
                </p>

                <h2>Step 2: Wire Module Exports</h2>
                <h3>
                    File: <code>app/src/contracts/lib.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub mod api;
pub mod validation;`}</code>
                </pre>
                <h3>
                    File: <code>app/src/contracts/api/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub mod v1;`}</code>
                </pre>
                <h3>
                    File: <code>app/src/contracts/api/v1/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub mod article;`}</code>
                </pre>
                <h3>
                    File: <code>app/src/contracts/validation/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub mod db;
pub mod sync;`}</code>
                </pre>

                <h2>Step 3: Write Request + Response DTO</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/article.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_db::platform::attachments::types::AttachmentUploadDto;
use generated::{localized::MultiLang, models::ArticleStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct ArticleMetaInput {
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub seo_title: String,
    #[validate(range(min = 0, max = 600))]
    #[schemars(range(min = 0, max = 600))]
    pub reading_minutes: i32,
    pub is_featured: bool,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct ArticleCreateInput {
    #[validate(range(min = 1))]
    #[schemars(range(min = 1))]
    pub category_id: i64,
    pub status: ArticleStatus,
    pub title: MultiLang,
    pub summary: MultiLang,
    #[validate(nested)]
    pub meta: ArticleMetaInput,
    pub cover: Option<AttachmentUploadDto>,
    pub galleries: Vec<AttachmentUploadDto>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ArticleCreateOutput {
    pub id: i64,
}`}</code>
                </pre>
                <p>
                    Request DTO: <code>Deserialize + Validate + JsonSchema</code>. Response DTO:{' '}
                    <code>Serialize + JsonSchema</code>.
                </p>

                <h2>Step 4: Add Reusable Sync Validators</h2>
                <p>
                    Avoid per-module validators like <code>validate_article_title</code> unless the
                    rule is truly domain-specific. Put generic validators in one reusable file.
                </p>
                <h3>
                    File: <code>app/src/contracts/validation/sync.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use std::borrow::Cow;
use validator::ValidationError;

fn err(code: &'static str, msg: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(msg))
}

pub fn required_alpha_dash_3_32(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value)?;
    core_web::rules::alpha_dash(value)?;
    if value.len() < 3 || value.len() > 32 {
        return Err(err("length_3_32", "Value must be 3-32 characters."));
    }
    Ok(())
}

pub fn required_slug(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value)?;
    core_web::rules::lowercase_slug(value)
}

pub fn strong_password(value: &str) -> Result<(), ValidationError> {
    core_web::rules::strong_password(value)
}`}</code>
                </pre>

                <h2>Step 5: Add Reusable DB Validation Helpers</h2>
                <p>
                    Reuse framework-level <code>Unique</code>, <code>Exists</code>,{' '}
                    <code>NotExists</code> through shared helper functions.
                </p>
                <h3>
                    File: <code>app/src/contracts/validation/db.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use anyhow::Result;
use core_web::rules::{AsyncRule, Exists, NotExists, Unique};

pub async fn ensure_unique(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    Unique::new(table, column, value).check(db).await
}

pub async fn ensure_exists(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    Exists::new(table, column, value).check(db).await
}

pub async fn ensure_not_exists(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    NotExists::new(table, column, value).check(db).await
}`}</code>
                </pre>

                <h2>Step 6: Apply Reusable Validators in DTO</h2>
                <h3>
                    File: <code>app/src/contracts/api/v1/admin_user.rs</code> (example)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};
use core_web::extract::AsyncValidate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminUserCreateInput {
    #[validate(custom(function = "crate::validation::sync::required_alpha_dash_3_32"))]
    #[schemars(length(min = 3, max = 32))]
    #[schemars(regex(pattern = "^[A-Za-z0-9_-]+$"))]
    #[schemars(description = "Letters, numbers, underscore, dash.")]
    pub username: String,

    #[validate(custom(function = "crate::validation::sync::strong_password"))]
    pub password: String,
}

#[async_trait::async_trait]
impl AsyncValidate for AdminUserCreateInput {
    async fn validate_async(&self, db: &sqlx::PgPool) -> anyhow::Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if !crate::validation::db::ensure_unique(db, "admin", "username", &self.username).await? {
            errors.add("username", ValidationError::new("unique"));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}`}</code>
                </pre>

                <h2>Step 7: Use DTO at Handler Boundary</h2>
                <h3>
                    File: <code>app/src/api/v1/article.rs</code> (sync example)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::extract::State;
use contracts::api::v1::article::{ArticleCreateInput, ArticleCreateOutput};
use core_i18n::t;
use core_web::{contracts::ContractJson, error::AppError, response::ApiResponse};

async fn create(
    State(state): State<AppApiState>,
    ContractJson(req): ContractJson<ArticleCreateInput>,
) -> Result<ApiResponse<ArticleCreateOutput>, AppError> {
    let _ = (state, req); // req already validated
    Ok(ApiResponse::success(ArticleCreateOutput { id: 1 }, &t("Article created")))
}`}</code>
                </pre>
                <h3>
                    File: <code>app/src/api/v1/admin_user.rs</code> (async example)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use contracts::api::v1::admin_user::AdminUserCreateInput;
use core_web::contracts::AsyncContractJson;

async fn create_admin_user(
    AsyncContractJson(req): AsyncContractJson<AdminUserCreateInput>,
) -> Result<ApiResponse<()>, AppError> {
    let _ = req; // sync + async validation already executed
    Ok(ApiResponse::success((), "ok"))
}`}</code>
                </pre>

                <h2>Step 8: OpenAPI Note for Custom Rules</h2>
                <p>
                    <code>#[validate(custom(function = "..."))]</code> is runtime-only. OpenAPI
                    cannot infer custom logic automatically.
                </p>
                <p>
                    Always add matching <code>#[schemars(...)]</code> hints (length/range/regex/
                    description/example) for the same field.
                </p>

                <h2>Step 9: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p contracts -p api-server
./console route list
curl -sS http://127.0.0.1:3000/openapi.json > /tmp/openapi.json`}</code>
                </pre>
            </div>
        </div>
    )
}
