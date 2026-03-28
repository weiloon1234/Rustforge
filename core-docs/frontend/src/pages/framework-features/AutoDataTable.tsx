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
                <h2>Scaffold Now (verified)</h2>
                <p>
                    This feature is wired by default for multiple admin datatable scopes in scaffold:
                    <code>admin.account</code>, <code>admin.content_page</code>,{' '}
                    <code>admin.country</code>, <code>admin.http_client_log</code>, and{' '}
                    <code>admin.webhook_log</code>. Additional scopes (for example merchant/article) are
                    <strong> Concept Extension (optional)</strong>.
                </p>

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
                        Add a new datatable in 3 edits: contract file, hooks file, and one
                        catalog entry in <code>app/src/internal/datatables/v1/admin/mod.rs</code>.
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
                    <li>
                        Cross-page totals are implemented as an optional summary endpoint
                        that reuses the same filter payload (example:
                        <code> /api/v1/admin/datatable/admin/summary</code>).
                    </li>
                    <li>
                        Shared React datatable emits <code>onPreCall</code> and{' '}
                        <code>onPostCall</code> with full filter snapshot (<code>all</code> and{' '}
                        <code>applied</code>) so each portal can hook analytics/custom behavior.
                    </li>
                    <li>
                        No generic <code>/dt</code> route and no client-controlled model
                        dispatch. Scoped routes bind the model key internally.
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
POST   /api/v1/admin/datatable/admin/summary
POST   /api/v1/admin/datatable/admin/export/csv
POST   /api/v1/admin/datatable/admin/export/email
GET    /api/v1/admin/datatable/admin/export/status?job_id=...

# Additional scaffold scopes (query/export)
POST   /api/v1/admin/datatable/http-client-log/query
POST   /api/v1/admin/datatable/webhook-log/query
POST   /api/v1/admin/datatable/content_page/query
POST   /api/v1/admin/datatable/country/query`}</code>
                </pre>

                <h2>Step 1: Contract SSOT (Row DTO + Filter Metadata)</h2>
                <p>
                    The contract file defines the Row DTO and filter UI. No custom request structs
                    needed &mdash; use the built-in generic types.
                </p>
                <p>
                    Type generation note: datatable row DTOs participate in the same{' '}
                    <code>make gen-types</code> scan as API DTOs. If a row field uses a generated/
                    framework enum directly, that enum is auto-added to the generated{' '}
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
use core_web::ids::SnowflakeId;
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.account";
pub const ROUTE_PREFIX: &str = "/datatable/admin";

/// Row DTO — the actual shape returned per record.
/// Excludes sensitive fields (password, deleted_at),
/// properly types abilities as Vec<String>.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableRow {
    pub id: SnowflakeId,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub created_at: String,
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

    fn scoped_key(&self) -> &'static str { SCOPED_KEY }
    fn openapi_tag(&self) -> &'static str { "Admin Account" }

    // Optional overrides only. Default trait implementations already map:
    // email_to_input / email_recipients / email_subject / export_file_name
    // from DataTableGenericEmailExportRequest.

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

                <h2>Step 2: Datatable Hooks (Scope, Filters, Typed Mapping)</h2>
                <p>
                    Hooks are defined per-model in{' '}
                    <code>app/src/internal/datatables/v1/admin/account.rs</code>. The generated hooks trait
                    provides <code>scope</code>, <code>authorize</code>,{' '}
                    <code>filter_query</code>, <code>filters</code>, <code>map_row</code>, and{' '}
                    <code>row_to_record</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/datatables/v1/admin/account.rs
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

    /// Typed post-fetch row transform. Mutate the app-facing row first.
    /// WithRelations implements Deref<Target=View>, so field access is transparent.
    fn map_row(
        &self, row: &mut AdminWithRelations,
        _input: &DataTableInput, _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        row.identity = row.identity();
        Ok(())
    }

    /// Optional final record projection for display/export parity.
    /// WithRelations serializes flat (via #[serde(flatten)]) — no "row" wrapper.
    fn row_to_record(
        &self, row: AdminWithRelations,
        _input: &DataTableInput, _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row)?;
        record.remove("password");
        record.remove("deleted_at");
        Ok(record)
    }
}`}</code>
                </pre>
                <p>
                    Query and CSV export now go through the same filter and mapping pipeline. Keep display/export
                    transformations on the typed row or in <code>row_to_record</code> so query results and CSV
                    output stay aligned.
                </p>

                <h3>Step 2B: Extra Summary Payload (Scaffold sample)</h3>
                <p>
                    Scaffold admin datatable provides a summary endpoint with{' '}
                    <code>total_admin_counts</code> and per-type counts. This runs on the
                    fully filtered query (no pagination limit) and returns a compact payload for
                    dashboard cards.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/datatables/v1/admin/account.rs
#[derive(Debug, Serialize)]
struct AdminDatatableSummary {
    total_admin_counts: i64,
    total_filtered: i64, // secondary total field for summary consumers
    developer_count: i64,
    superadmin_count: i64,
    admin_count: i64,
}

async fn build_admin_summary(
    db: &sqlx::PgPool,
    input: &DataTableInput,
    ctx: &DataTableContext,
) -> anyhow::Result<serde_json::Value> {
    let scoped = apply_actor_scope(Admin::new(db, None).query(), ctx);
    let filtered = apply_summary_filters(scoped, input);

    let total_filtered = filtered.clone().count(db).await?;
    let developer_count = filtered.clone().where_admin_type(Op::Eq, AdminType::Developer).count(db).await?;
    let superadmin_count = filtered.clone().where_admin_type(Op::Eq, AdminType::SuperAdmin).count(db).await?;
    let admin_count = filtered.where_admin_type(Op::Eq, AdminType::Admin).count(db).await?;

    Ok(serde_json::to_value(AdminDatatableSummary {
        total_admin_counts: total_filtered,
        total_filtered,
        developer_count,
        superadmin_count,
        admin_count,
    })?)
}`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-text">{`POST /api/v1/admin/datatable/admin/summary
Body:
{
  "base": { "include_meta": false },
  "q": "john",
  "f-admin_type": "admin"
}`}</code>
                </pre>
                <p>
                    Put complex aggregates in the same scoped datatable module where filtering/scope
                    rules already live (for scaffold: <code>app/src/internal/datatables/v1/admin/account.rs</code>).
                    Build the aggregate query from the same filtered base query to keep totals consistent.
                </p>

                <h2>Step 3: Single Catalog Register + Mount</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/internal/datatables/v1/admin/mod.rs
pub static ADMIN_SCOPED_DATATABLES: &[ScopedDatatableSpec] = &[
    ScopedDatatableSpec {
        scoped_key: "admin.account",
        route_prefix: "/datatable/admin",
        register: account::register_scoped,
        mount_routes: account::routes,
    },
    // ...http_client_log, webhook_log, content_page...
];

// app/src/internal/api/state.rs
crate::internal::datatables::v1::admin::register_scoped_datatables(
    &mut datatable_registry,
    &ctx.db,
);

// app/src/internal/api/datatable.rs
crate::internal::datatables::v1::admin::mount_scoped_datatable_routes(state.clone());`}</code>
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
  ├─ 7. fetch typed rows
  ├─ 8. map_row()            → typed row transform
  ├─ 9. row_to_record()      → optional JSON projection
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

                <h3>Summary Response Shape (Optional Extension Endpoint)</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-json">{`{
  "data": {
    "total_admin_counts": 42,
    "developer_count": 1,
    "superadmin_count": 6,
    "admin_count": 35
  },
  "message": "datatable summary"
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

                <h2>Frontend Consumption Pattern</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-ts">{`<DataTableApiProvider api={api}>
  <DataTable<AdminDatatableRow>
    url="/api/v1/admin/datatable/admin/query"
    title="Admins"
    subtitle="Manage administrator accounts"
    columns={[
      { key: "actions", label: "Actions", sortable: false, render: (record, ctx) => (
          <button onClick={() => openEditModal(record, ctx.refresh)}>Edit</button>
        ),
      },
      { key: "username", label: "Username", render: (record) => record.username },
      { key: "email", label: "Email", render: (record) => record.email ?? "—" },
      { key: "created_at", label: "Created At", render: (record) => formatDateTime(record.created_at) },
    ]}
    // # index column is auto-enabled by default
    showIndexColumn
    // rowKey optional; defaults to record.id when available
    // rowKey={(record) => String(record.id)}
    onPreCall={(event) => {
      // event.filters.all includes every filter key with current value
      // event.filters.applied includes only non-empty filters
    }}
  onPostCall={(event) => {
    if (!event.response) return;
    void api.post("/api/v1/admin/datatable/admin/summary", {
      base: { include_meta: false },
      ...event.filters.applied,
    }).then((res) => {
      const total = res.data?.data?.total_admin_counts ?? 0;
      console.log("filtered total", total);
    });
  }}
  renderTableFooter={({ records }) => (
    <tr>
      <td colSpan={99}>Page rows: {records.length}</td>
    </tr>
  )}
  />
</DataTableApiProvider>`}</code>
                </pre>
                <p>
                    Frontend renderers now return inner content only (<code>span/div/text</code>);
                    the shared datatable wraps cells with consistent <code>&lt;td&gt;</code> structure
                    and classes.
                </p>
                <h3>Page Footer Metrics vs Cross-Page Summary</h3>
                <ul>
                    <li>
                        <strong>Page footer metrics</strong> (for example <code>records.length</code>,{' '}
                        <code>sumColumn("amount")</code>) are client-side and only reflect the
                        currently loaded page.
                    </li>
                    <li>
                        <strong>Cross-page totals/cards</strong> must come from a backend aggregate
                        query using the same applied filters and scope (example:
                        <code> /api/v1/admin/datatable/admin/summary</code>).
                    </li>
                    <li>
                        If you need both, use footer metrics for quick per-page context and a
                        dedicated backend summary payload for business totals.
                    </li>
                </ul>

                <h2>Typed mapping and export parity</h2>
                <p>
                    Query and CSV export should go through the same filter and mapping pipeline. Keep typed row
                    changes in <code>map_row</code> and final JSON/CSV projection changes in <code>row_to_record</code>.
                </p>
                <ul>
                    <li><code>map_row</code>: mutate the typed row before record materialization.</li>
                    <li><code>row_to_record</code>: final record projection for query response and CSV export.</li>
                    <li>Do not keep one display mapping path and a separate raw export path.</li>
                </ul>

                <h2>Frontend export column control</h2>
                <p>
                    The React datatable sends visible export headers to the backend, so the page-level column definition is the export SSOT as well.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-tsx">{`columns={[
  { key: 'actions', label: 'Actions', exportIgnore: true },
  { key: 'title', label: 'Title' },
  {
    key: 'is_system',
    label: 'System',
    exportColumn: 'is_system_explained',
    render: (row) => row.is_system_explained,
  },
]}
exportIgnoreColumns={['id']}`}</code>
                </pre>
                <p>
                    Use <code>exportColumn</code> when the visible UI field should export a different backend field, and use <code>exportIgnore</code> or <code>exportIgnoreColumns</code> for action columns or page-specific exclusions.
                </p>

                <h2>Generated vs custom datatables</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Use the generated path when</th>
                            <th>Use a custom runtime when</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Filters and sorting are mostly column-driven.</td>
                            <td>The source table is not backed by an app model source or the query shape is unusually custom.</td>
                        </tr>
                        <tr>
                            <td>You can express access rules with generated query hooks.</td>
                            <td>You need special joins, summary-only rows, or a framework-owned source without an app model-source contract.</td>
                        </tr>
                        <tr>
                            <td>You want generator-owned row/view types and table adapters.</td>
                            <td>You still need the datatable contract surface, but the runtime query cannot come from a normal generated adapter.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Recipe handoff</h2>
                <ul>
                    <li><a href="#/cookbook/add-admin-datatable">Add an Admin DataTable</a> for the step-by-step starter recipe.</li>
                    <li><a href="#/cookbook/build-end-to-end-flow">Build an End-to-End Flow</a> for a full vertical slice using the same pieces together.</li>
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

# summary cards (cross-page totals)
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/summary \\
  -H 'Authorization: Bearer <TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{"base":{"include_meta":false},"f-admin_type":"admin"}'

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
