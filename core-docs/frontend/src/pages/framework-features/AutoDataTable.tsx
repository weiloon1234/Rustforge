import { useEffect } from 'react'
import Prism from 'prismjs'

export function AutoDataTableFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">AutoDataTable</h1>
                <p className="text-xl text-gray-500">
                    DTO-first scoped datatable contract with strong OpenAPI and one SSOT file per
                    scoped model.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Latest Standard</h2>
                <ul>
                    <li>
                        Scope key is explicit (example: <code>admin.article</code>,{' '}
                        <code>merchant.article</code>).
                    </li>
                    <li>
                        One SSOT file per scoped model at{' '}
                        <code>app/src/contracts/datatable/&lt;scope&gt;/&lt;model&gt;.rs</code>.
                    </li>
                    <li>
                        Datatable endpoints are JSON-first and simplified to 4 routes only.
                    </li>
                    <li>
                        First request pattern: <code>include_meta=true</code> (default) returns
                        records and frontend metadata in one response.
                    </li>
                    <li>
                        Filter layout metadata uses nested rows: <code>filter_rows: Vec&lt;Vec&lt;...&gt;&gt;</code>.
                    </li>
                </ul>

                <h2>Routes</h2>
                <ul>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/query</code>
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/export/csv</code>
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/export/email</code>
                    </li>
                    <li>
                        <code>GET /api/v1/admin/datatable/admin/export/status?job_id=...</code>
                    </li>
                </ul>

                <h2>Step 1: Scoped Contract SSOT</h2>
                <p>
                    Example contract file for admin portal datatable. This file owns request DTO,
                    row DTO type, filter metadata rows, and request-to-query mapping.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/contracts/datatable/admin/article.rs
use std::collections::BTreeMap;

use core_datatable::DataTableInput;
use core_web::datatable::{
    DataTableEmailExportRequestBase, DataTableFilterFieldDto, DataTableFilterFieldType,
    DataTableFilterOptionDto, DataTableQueryRequestBase, DataTableScopedContract,
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
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub created_at_from: Option<String>,
    #[serde(default)]
    pub created_at_to: Option<String>,
}

impl ArticleDatatableQueryInput {
    pub fn to_input(&self) -> DataTableInput {
        let mut input = self.base.to_input();
        let mut params = BTreeMap::new();

        if let Some(status) = self.status {
            params.insert("f-status".to_string(), status.as_str().to_string());
        }
        if let Some(title) = self.title.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            params.insert("f-locale-like-title".to_string(), title.to_string());
        }
        if let Some(from) = self.created_at_from.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            params.insert("f-date-from-created_at".to_string(), from.to_string());
        }
        if let Some(to) = self.created_at_to.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            params.insert("f-date-to-created_at".to_string(), to.to_string());
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

    fn email_subject(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.subject.clone()
    }

    fn export_file_name(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.export_file_name.clone()
    }

    fn include_meta(&self, req: &Self::QueryRequest) -> bool {
        req.base.include_meta
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![DataTableFilterFieldDto {
                field: "status".to_string(),
                filter_key: "f-status".to_string(),
                field_type: DataTableFilterFieldType::Select,
                label: "Status".to_string(),
                placeholder: Some("Select status".to_string()),
                description: None,
                options: Some(
                    ArticleStatus::variants()
                        .iter()
                        .map(|v| DataTableFilterOptionDto {
                            label: v.as_str().to_string(),
                            value: v.as_str().to_string(),
                        })
                        .collect(),
                ),
            }],
        ]
    }
}`}</code>
                </pre>

                <h2>Step 2: Register Scoped Key + Mount Routes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/api/state.rs
let mut datatable_registry = DataTableRegistry::new();
crate::internal::datatables::register_all_generated_datatables(&mut datatable_registry, &ctx.db);
datatable_registry.register_as(
    "admin.article",
    crate::internal::datatables::app_article_datatable(ctx.db.clone()),
);

// app/src/internal/api/datatable.rs
core_web::datatable::routes_for_scoped_contract_with_options(
    "/datatable/admin/articles",
    state,
    AdminArticleDatatableContract::default(),
    DataTableRouteOptions { require_bearer_auth: true },
)`}</code>
                </pre>

                <h2>Step 3: Response Contract</h2>
                <p>
                    Query returns a typed envelope with row DTOs plus paging/diagnostics. Metadata
                    is optional (controlled by <code>include_meta</code>).
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-json">{`{
  "data": {
    "records": [ ...row_dto... ],
    "per_page": 30,
    "total_records": 120,
    "total_pages": 4,
    "page": 1,
    "pagination_mode": "offset",
    "has_more": false,
    "next_cursor": null,
    "diagnostics": {
      "duration_ms": 12,
      "auto_filters_applied": 1,
      "unknown_filters": [],
      "unknown_filter_mode": "warn"
    },
    "meta": {
      "model_key": "admin.article",
      "defaults": { ... },
      "columns": [ ... ],
      "relation_columns": [ ... ],
      "filter_rows": [
        [ { "field": "status", "type": "select", "options": [ ... ] } ],
        [ { "field": "created_at_from", "type": "datetime" }, { "field": "created_at_to", "type": "datetime" } ]
      ]
    }
  },
  "message": "datatable query"
}`}</code>
                </pre>

                <h2>Defaults</h2>
                <ul>
                    <li>
                        <code>include_meta</code> default is <code>true</code>.
                    </li>
                    <li>
                        If model has <code>created_at</code>, default range filters are auto-injected
                        unless contract disables it.
                    </li>
                    <li>
                        CSV email export is link-only. File is uploaded to storage and emailed as a
                        presigned URL.
                    </li>
                    <li>
                        Presigned URL TTL defaults to 7 days via{' '}
                        <code>DATATABLE_EXPORT_LINK_TTL_SECS</code>.
                    </li>
                </ul>

                <h2>OpenAPI + Route List</h2>
                <ul>
                    <li>OpenAPI request/response shape comes from your scoped request/row DTO.</li>
                    <li>
                        Operation IDs are stable and scoped by route prefix + scoped model key.
                    </li>
                    <li>
                        <code>./console route list --json</code> can read the simplified datatable
                        routes directly.
                    </li>
                </ul>

                <h2>Curl Quick Check</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# query with first-hit metadata
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/query \
  -H 'Authorization: Bearer <TOKEN>' \
  -H 'Content-Type: application/json' \
  -d '{"base":{"include_meta":true,"page":1,"per_page":30}}'

# queue email export (link-only)
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/email \
  -H 'Authorization: Bearer <TOKEN>' \
  -H 'Content-Type: application/json' \
  -d '{"base":{"query":{"include_meta":false},"recipients":["ops@example.com"]}}'

# poll status
curl 'http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/status?job_id=<JOB_ID>' \
  -H 'Authorization: Bearer <TOKEN>'`}</code>
                </pre>
            </div>
        </div>
    )
}
