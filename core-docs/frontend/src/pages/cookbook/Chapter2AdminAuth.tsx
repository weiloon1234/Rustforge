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
                    Chapter 2: DTO + Validation + Datatable Contract SSOT
                </h1>
                <p className="text-xl text-gray-500">
                    Extend Chapter 1 with centralized DTO files, reusable validation rules, and the
                    same SSOT pattern for datatable routes.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope</h2>
                <ul>
                    <li>
                        API request/response DTO source of truth: <code>app/src/contracts/api/*</code>.
                    </li>
                    <li>
                        Datatable contract source of truth:{' '}
                        <code>app/src/contracts/datatable/&lt;scope&gt;/&lt;model&gt;.rs</code>.
                    </li>
                    <li>
                        Reusable validation rules: <code>app/src/validation/*</code>.
                    </li>
                    <li>
                        Runtime validation from <code>ValidatedJson</code>/<code>AsyncValidatedJson</code>,
                        OpenAPI constraints from <code>JsonSchema</code>/<code>schemars</code>.
                    </li>
                </ul>

                <h2>Step 1: Folder Layout</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`app/src/contracts/mod.rs
app/src/contracts/api/mod.rs
app/src/contracts/api/v1/mod.rs
app/src/contracts/api/v1/article.rs
app/src/contracts/datatable/mod.rs
app/src/contracts/datatable/admin/mod.rs
app/src/contracts/datatable/admin/article.rs
app/src/validation/mod.rs
app/src/validation/sync.rs
app/src/validation/db.rs`}</code>
                </pre>

                <h2>Step 2: API DTO Example</h2>
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

                <h2>Step 3: Reusable Validation Rules</h2>
                <h3>
                    File: <code>app/src/validation/sync.rs</code>
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
}`}</code>
                </pre>

                <h3>
                    File: <code>app/src/validation/db.rs</code>
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

                <h2>Step 4: Handler Boundary Rule</h2>
                <p>
                    After <code>ContractJson&lt;T&gt;</code> (or <code>AsyncContractJson&lt;T&gt;</code>)
                    extraction succeeds, <code>req</code> is already validated and can be used
                    directly in workflow code.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::extract::State;
use core_i18n::t;
use core_web::{contracts::ContractJson, error::AppError, response::ApiResponse};

use crate::contracts::api::v1::article::{ArticleCreateInput, ArticleCreateOutput};

async fn create(
    State(state): State<AppApiState>,
    req: ContractJson<ArticleCreateInput>,
) -> Result<ApiResponse<ArticleCreateOutput>, AppError> {
    let req = req.0; // already validated
    let _ = state;

    Ok(ApiResponse::success(
        ArticleCreateOutput { id: 1 },
        &t("Article created"),
    ))
}`}</code>
                </pre>

                <h2>Step 5: Datatable Contract SSOT (Same DTO Principle)</h2>
                <h3>
                    File: <code>app/src/contracts/datatable/admin/article.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use std::collections::BTreeMap;

use core_datatable::DataTableInput;
use core_web::datatable::{
    DataTableEmailExportRequestBase, DataTableQueryRequestBase, DataTableScopedContract,
};
use generated::models::{ArticleStatus, ArticleView};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct ArticleDatatableQueryInput {
    #[validate(nested)]
    pub base: DataTableQueryRequestBase,
    #[serde(default)]
    pub status: Option<ArticleStatus>,
}

impl ArticleDatatableQueryInput {
    pub fn to_input(&self) -> DataTableInput {
        let mut input = self.base.to_input();
        let mut params = BTreeMap::new();

        if let Some(status) = self.status {
            params.insert("f-status".to_string(), status.as_str().to_string());
        }

        input.params.extend(params);
        input
    }
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct ArticleDatatableEmailExportInput {
    #[validate(nested)]
    pub base: DataTableEmailExportRequestBase,
    #[serde(default)]
    pub status: Option<ArticleStatus>,
}

#[derive(Debug, Clone, Default)]
pub struct AdminArticleDatatableContract;

impl DataTableScopedContract for AdminArticleDatatableContract {
    type QueryRequest = ArticleDatatableQueryInput;
    type EmailRequest = ArticleDatatableEmailExportInput;
    type Row = ArticleView;

    fn scoped_key(&self) -> &'static str {
        "admin.article"
    }

    fn query_to_input(&self, req: &Self::QueryRequest) -> DataTableInput {
        req.to_input()
    }

    fn email_to_input(&self, req: &Self::EmailRequest) -> DataTableInput {
        let mut input = req.base.query.to_input();
        if let Some(status) = req.status {
            input.params.insert("f-status".to_string(), status.as_str().to_string());
        }
        input.export_file_name = req.base.export_file_name.clone();
        input
    }

    fn email_recipients(&self, req: &Self::EmailRequest) -> Vec<String> {
        req.base.recipients.clone()
    }
}`}</code>
                </pre>

                <h2>Step 6: Mount Contract Routes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/api/datatable.rs
core_web::datatable::routes_for_scoped_contract_with_options(
    "/datatable/admin/articles",
    state,
    AdminArticleDatatableContract::default(),
    core_web::datatable::DataTableRouteOptions {
        require_bearer_auth: true,
    },
)`}</code>
                </pre>

                <h2>Step 7: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check --workspace
./console route list
curl -sS http://127.0.0.1:3000/openapi.json > /tmp/openapi.json`}</code>
                </pre>
            </div>
        </div>
    )
}
