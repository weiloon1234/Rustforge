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
                    Meta-driven scoped datatable with auto-filters, typed Row DTOs, and zero-boilerplate
                    request handling.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Key Concepts</h2>
                <ul>
                    <li>
                        Scope key is explicit (example: <code>admin.article</code>,{' '}
                        <code>merchant.article</code>).
                    </li>
                    <li>
                        One SSOT contract file per scoped model at{' '}
                        <code>app/src/contracts/datatable/&lt;scope&gt;/&lt;model&gt;.rs</code>.
                    </li>
                    <li>
                        <strong>No per-datatable request structs needed.</strong>{' '}
                        <code>DataTableGenericQueryRequest</code> and{' '}
                        <code>DataTableGenericEmailExportRequest</code> handle all filter params
                        automatically via <code>#[serde(flatten)]</code>.
                    </li>
                    <li>
                        First request pattern: <code>include_meta=true</code> (default) returns
                        records and frontend metadata (columns, filter UI) in one response.
                    </li>
                    <li>
                        Filter layout metadata uses nested rows:{' '}
                        <code>filter_rows: Vec&lt;Vec&lt;DataTableFilterFieldDto&gt;&gt;</code>.
                    </li>
                </ul>

                <h2>Three-Tier Filter Pipeline</h2>
                <p>
                    Every filter param goes through the pipeline in order. The{' '}
                    <code>filter_key</code> in <code>DataTableFilterFieldDto</code> determines
                    which tier handles it.
                </p>
                <table>
                    <thead>
                        <tr>
                            <th>Key prefix</th>
                            <th>Handler</th>
                            <th>When to use</th>
                            <th>Fallback</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><code>f-*</code></td>
                            <td><code>apply_auto_filter</code> (generated)</td>
                            <td>Simple column filters: <code>f-like-email</code>, <code>f-admin_type</code>, <code>f-date-from-created_at</code></td>
                            <td><code>unknown_filters</code></td>
                        </tr>
                        <tr>
                            <td>non-<code>f-</code></td>
                            <td><code>filter_query</code> hook (your impl)</td>
                            <td>Custom/multi-column queries: <code>q</code> (keyword search), complex joins</td>
                            <td><code>unknown_filters</code></td>
                        </tr>
                        <tr>
                            <td>&mdash;</td>
                            <td><code>filters()</code> hook</td>
                            <td>Cross-field business logic, always-on security filters</td>
                            <td>&mdash;</td>
                        </tr>
                    </tbody>
                </table>

                <h3>Auto-filter keys (<code>f-</code> prefix)</h3>
                <p>
                    These are parsed by <code>parse_filter_key()</code> and applied by the
                    generated adapter. No hook code needed.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`f-<col>                        Exact match       WHERE col = val
f-like-<col>                   Contains           WHERE col LIKE %val%
f-gte-<col>                    Greater or equal   WHERE col >= val
f-lte-<col>                    Less or equal      WHERE col <= val
f-date-from-<col>              Date range start   WHERE col >= parsed_date
f-date-to-<col>                Date range end     WHERE col <= parsed_date
f-like-any-<col1|col2|...>     Multi-col search   WHERE (c1 LIKE %val% OR c2 LIKE %val%)
f-any-<col1|col2|...>          Multi-col exact    WHERE (c1 = val OR c2 = val)
f-has-<relation>-<col>         Related model      WHERE EXISTS(SELECT 1 FROM rel WHERE rel.col = val)
f-has-like-<relation>-<col>    Related model LIKE WHERE EXISTS(SELECT 1 FROM rel WHERE rel.col LIKE %val%)
f-locale-<col>                 Localized exact    Via localization table
f-locale-like-<col>            Localized LIKE     Via localization table`}</code>
                </pre>

                <h2>Routes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`POST   /api/v1/admin/datatable/admin/query
POST   /api/v1/admin/datatable/admin/export/csv
POST   /api/v1/admin/datatable/admin/export/email
GET    /api/v1/admin/datatable/admin/export/status?job_id=...`}</code>
                </pre>

                <h2>Step 1: Contract SSOT (Row DTO + Filter Metadata)</h2>
                <p>
                    The contract file defines the Row DTO and filter UI. No custom request structs
                    needed &mdash; use the built-in generic types.
                </p>
                <p>
                    Type generation note: datatable row DTOs participate in the same{' '}
                    <code>make gen-types</code> scan as API DTOs. If a row field uses an enum via{' '}
                    <code>#[ts(type = "EnumName")]</code>, that enum is auto-added to the generated{' '}
                    <code>admin/types/enums.ts</code> only when referenced.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/contracts/datatable/admin/account.rs
use core_datatable::DataTableInput;
use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest,
    DataTableScopedContract,
};
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Row DTO — the actual shape returned per record.
/// Excludes sensitive fields (password, deleted_at),
/// properly types abilities as Vec<String>.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableRow {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    #[ts(type = "string[]")]
    pub abilities: Vec<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminAdminDataTableContract;

impl DataTableScopedContract for AdminAdminDataTableContract {
    // Generic types — no per-datatable request structs needed.
    // filter_key values from filter_rows() are sent directly by
    // the frontend and forwarded to auto-filter / filter_query.
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = AdminDatatableRow;

    fn scoped_key(&self) -> &'static str { "admin.account" }
    fn openapi_tag(&self) -> &'static str { "Admin Account" }

    fn email_to_input(&self, req: &Self::EmailRequest) -> DataTableInput { req.to_input() }
    fn email_recipients(&self, req: &Self::EmailRequest) -> Vec<String> { req.base.recipients.clone() }
    fn email_subject(&self, req: &Self::EmailRequest) -> Option<String> { req.base.subject.clone() }
    fn export_file_name(&self, req: &Self::EmailRequest) -> Option<String> { req.base.export_file_name.clone() }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),                   // non-f- → filter_query hook
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search name/username/email".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "email".to_string(),
                    filter_key: "f-like-email".to_string(),        // f- → auto-filter
                    field_type: DataTableFilterFieldType::Text,
                    label: "Email".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![DataTableFilterFieldDto {
                field: "admin_type".to_string(),
                filter_key: "f-admin_type".to_string(),            // f- → auto-filter
                field_type: DataTableFilterFieldType::Select,
                label: "Admin Type".to_string(),
                placeholder: Some("Choose type".to_string()),
                description: None,
                options: Some(AdminType::datatable_filter_options()),
            }],
        ]
    }
}`}</code>
                </pre>

                <h2>Step 2: Datatable Hooks (Scope, Filters, Mappings)</h2>
                <p>
                    Hooks are defined per-model in{' '}
                    <code>app/src/internal/datatables/portal/admin/account.rs</code>. The generated hooks trait
                    provides <code>scope</code>, <code>authorize</code>,{' '}
                    <code>filter_query</code>, <code>filters</code>, and <code>mappings</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/datatables/portal/admin/account.rs
use core_datatable::{DataTableContext, DataTableInput};
use core_db::common::sql::Op;
use generated::models::{AdminCol, AdminDataTableHooks, AdminQuery, AdminType};

pub struct AdminDataTableAppHooks;

impl AdminDataTableHooks for AdminDataTableAppHooks {
    /// Row-level security — runs before all filters.
    fn scope<'db>(
        &'db self, query: AdminQuery<'db>,
        _input: &DataTableInput, ctx: &DataTableContext,
    ) -> AdminQuery<'db> {
        let admin_type = ctx.actor.as_ref()
            .and_then(|a| a.attributes.get("admin_type"))
            .and_then(|v| v.as_str());
        match admin_type {
            Some("developer") => query,
            Some("superadmin") => query.where_admin_type(Op::Ne, AdminType::Developer),
            _ => query.where_id(Op::Eq, -1),
        }
    }

    /// Custom filter for non-f- keys. Receives filter_key + value.
    /// Return Ok(Some(query)) to apply, Ok(None) to skip → unknown_filters.
    fn filter_query<'db>(
        &'db self, query: AdminQuery<'db>,
        filter_key: &str, value: &str,
        _input: &DataTableInput, _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<AdminQuery<'db>>> {
        match filter_key {
            "q" => {
                let pattern = format!("%{value}%");
                Ok(Some(query.where_group(|q| {
                    q.where_col(AdminCol::Username, Op::Like, pattern.clone())
                        .or_where_col(AdminCol::Name, Op::Like, pattern.clone())
                        .or_where_col(AdminCol::Email, Op::Like, pattern)
                })))
            }
            _ => Ok(None),
        }
    }

    /// Post-fetch row transformation: strip sensitive fields,
    /// transform abilities from JSON array to Vec<String>.
    fn mappings(
        &self, record: &mut serde_json::Map<String, serde_json::Value>,
        _input: &DataTableInput, _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        record.remove("password");
        record.remove("deleted_at");
        // ...transform abilities...
        Ok(())
    }
}`}</code>
                </pre>

                <h2>Step 3: Register + Mount Routes</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/api/state.rs
let mut datatable_registry = DataTableRegistry::new();
crate::internal::datatables::register_all_generated_datatables(&mut datatable_registry, &ctx.db);

// app/src/internal/api/datatable.rs
core_web::datatable::routes_for_scoped_contract_with_options(
    "/datatable/admin",
    state,
    AdminAdminDataTableContract::default(),
    DataTableRouteOptions { require_bearer_auth: true },
)`}</code>
                </pre>

                <h2>Hook Execution Order</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`Request arrives
  │
  ├─ 1. authorize()          → permission check (can reject)
  ├─ 2. scope()              → base query scoping (row-level security)
  ├─ 3. auto-filter loop     → f-* params via generated adapter
  ├─ 4. filter_query loop    → non-f-* params via your hook
  ├─ 5. filters()            → catch-all custom query logic
  ├─ 6. sort + paginate
  ├─ 7. fetch rows
  ├─ 8. mappings()           → per-row JSON transform
  │
  └─ Response (records + diagnostics + meta)`}</code>
                </pre>

                <h2>Response Shape</h2>
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
      "model_key": "admin.account",
      "defaults": { "sorting_column": "id", "sorted": "desc", "per_page": 30 },
      "columns": [ { "name": "username", "label": "Username", "sortable": true, ... } ],
      "filter_rows": [
        [
          { "field": "q", "filter_key": "q", "type": "text", "label": "Keyword" },
          { "field": "email", "filter_key": "f-like-email", "type": "text", "label": "Email" }
        ],
        [
          { "field": "admin_type", "filter_key": "f-admin_type", "type": "select", "label": "Admin Type", "options": [...] }
        ]
      ]
    }
  },
  "message": "datatable query"
}`}</code>
                </pre>

                <h2>Defaults</h2>
                <ul>
                    <li>
                        <code>include_meta</code> default is <code>true</code>. Frontend caches
                        after first request.
                    </li>
                    <li>
                        If model has <code>created_at</code>, default date-range filters are
                        auto-injected unless contract disables them.
                    </li>
                    <li>
                        CSV email export uploads to storage and sends a presigned URL. TTL
                        defaults to 7 days via{' '}
                        <code>DATATABLE_EXPORT_LINK_TTL_SECS</code>.
                    </li>
                </ul>

                <h2>Curl Quick Check</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# query with metadata + keyword search
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/query \\
  -H 'Authorization: Bearer <TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{"base":{"include_meta":true,"page":1,"per_page":30},"q":"john"}'

# query with auto-filter
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/query \\
  -H 'Authorization: Bearer <TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{"base":{"page":1},"f-admin_type":"admin","f-like-email":"@example.com"}'

# queue email export
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/email \\
  -H 'Authorization: Bearer <TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{"base":{"query":{"include_meta":false},"recipients":["ops@example.com"]}}'

# poll export status
curl 'http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/status?job_id=<JOB_ID>' \\
  -H 'Authorization: Bearer <TOKEN>'`}</code>
                </pre>
            </div>
        </div>
    )
}
