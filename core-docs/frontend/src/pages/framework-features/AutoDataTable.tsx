export function AutoDataTableFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">AutoDataTable</h1>
                <p className="text-xl text-gray-500">
                    Framework-level datatable route collection with generated typed adapters and
                    OpenAPI-ready JSON/form contracts.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Latest Architecture (Single Pattern)</h2>
                <ul>
                    <li>
                        Schema stays the single source of truth in <code>app/schemas/*.toml</code>.
                    </li>
                    <li>
                        <code>db-gen</code> generates typed datatable adapters and app hook stubs.
                    </li>
                    <li>
                        App-level per-model hooks are first-class extension points:{' '}
                        <code>scope</code>, <code>authorize</code>, <code>filters</code>, and{' '}
                        <code>mappings</code>.
                    </li>
                    <li>
                        Framework provides reusable route collection in{' '}
                        <code>core_web::datatable</code>.
                    </li>
                    <li>
                        App level owns auth/middleware and route prefix policy (for example{' '}
                        <code>/api/v1/admin</code>).
                    </li>
                    <li>
                        OpenAPI schemas are generated from DTO extractors for query/json/form
                        endpoints.
                    </li>
                </ul>

                <h2>Enable In App (Admin Example)</h2>

                <h3>Step 1: Generate code from schema</h3>
                <p>
                    Keep datatable definitions in schema-driven models, then regenerate. Do not
                    edit generated files manually.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo check -p generated
# or make gen`}</code>
                </pre>

                <h3>Step 2: Customize app hook (optional, recommended)</h3>
                <p>
                    Generated app hooks live in <code>app/src/internal/datatables</code>. Use hook
                    methods for app-specific query scope, permission checks, custom filters, and
                    output transformation.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// app/src/internal/datatables/admin.rs
use anyhow::Context;
use core_datatable::{DataTableContext, DataTableInput};
use core_db::common::sql::{Op, RawClause, RawJoinSpec};
use generated::models::{AdminDataTable, AdminDataTableHooks, AdminQuery};
use serde_json::Value;

#[derive(Default, Clone)]
pub struct AdminDataTableAppHooks;

impl AdminDataTableHooks for AdminDataTableAppHooks {
    fn authorize(
        &self,
        _input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> anyhow::Result<bool> {
        Ok(ctx
            .actor
            .as_ref()
            .map(|a| a.has_role("developer") || a.has_permission("admin.read"))
            .unwrap_or(false))
    }

    fn scope<'db>(
        &'db self,
        mut query: AdminQuery<'db>,
        input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> AdminQuery<'db> {
        // base scope override for this model
        if let Some(admin_type) = input.params.get("f-admin-type").map(String::as_str) {
            query = query.where_admin_type_raw(Op::Eq, admin_type.to_string());
        }

        // optional safe raw fragment (bind-aware)
        if let Some(keyword) = input.params.get("q").map(|v| v.trim()).filter(|v| !v.is_empty()) {
            if let Ok(clause) = RawClause::new(
                "LOWER(admin.name) LIKE LOWER(?)",
                [format!("%{}%", keyword)],
            ) {
                query = query.unsafe_sql().where_raw(clause).done();
            }
        }

        query
    }

    fn filters<'db>(
        &'db self,
        mut query: AdminQuery<'db>,
        input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<AdminQuery<'db>> {
        // app-specific custom filter key
        if let Some(email_domain) = input.params.get("f-email-domain").map(|s| s.trim()) {
            if !email_domain.is_empty() {
                let clause = RawClause::new(
                    "admin.email ILIKE ?",
                    [format!("%@{}", email_domain)],
                )
                .context("invalid f-email-domain filter")?;
                query = query.unsafe_sql().where_raw(clause).done();
            }
        }

        // optional custom join when needed
        if matches!(input.params.get("with_recent_tokens").map(String::as_str), Some("1")) {
            let on = RawClause::new("pat.tokenable_id = admin.id", Vec::<String>::new())?;
            let join = RawJoinSpec::left("personal_access_tokens pat", on)?;
            query = query.unsafe_sql().join_raw(join).done();
        }

        Ok(query)
    }

    fn mappings(
        &self,
        record: &mut serde_json::Map<String, Value>,
        _input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        // column transformer / hide / computed flags
        record.remove("password");
        let can_delete = ctx
            .actor
            .as_ref()
            .map(|a| a.has_permission("admin.delete"))
            .unwrap_or(false);
        record.insert("can_delete".to_string(), Value::Bool(can_delete));
        Ok(())
    }
}

pub fn app_admin_datatable(db: sqlx::PgPool) -> AdminDataTable<AdminDataTableAppHooks> {
    AdminDataTable::new(db).with_hooks(AdminDataTableAppHooks::default())
}`}</code>
                </pre>

                <h3>Step 3: Build app datatable state wrapper</h3>
                <p>
                    App wrapper registers generated datatables and resolves actor from token. This
                    keeps framework generic while app controls policy enrichment.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// app/src/api/datatable.rs
#[derive(Clone)]
pub struct DataTableApiState {
    registry: Arc<DataTableRegistry>,
    async_exports: Arc<DataTableAsyncExportManager>,
    db: sqlx::PgPool,
}

impl DataTableApiState {
    pub fn new(ctx: &bootstrap::boot::BootContext) -> Self {
        let mut registry = DataTableRegistry::new();
        models::register_all_generated_datatables(&mut registry, &ctx.db);
        let registry = Arc::new(registry);
        let async_exports = Arc::new(DataTableAsyncExportManager::new(registry.clone()));
        Self { registry, async_exports, db: ctx.db.clone() }
    }
}

#[async_trait]
impl DataTableRouteState for DataTableApiState {
    fn datatable_registry(&self) -> &Arc<DataTableRegistry> { &self.registry }
    fn datatable_async_exports(&self) -> &Arc<DataTableAsyncExportManager> { &self.async_exports }

    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext {
        let actor = if let Some(token) = extract_request_token(headers) {
            models::datatable_actor::resolve_datatable_actor_from_token(&self.db, &token).await
        } else {
            None
        };
        DataTableContext {
            actor,
            ..Default::default()
        }
    }
}`}</code>
                </pre>

                <h3>Step 4: Mount model-bound route collection under /api/v1/admin/datatable/admin</h3>
                <p>
                    Assume admin middleware already exists at app level. Apply it when mounting the
                    framework route collection.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use axum::middleware::from_fn_with_state;
use core_web::datatable::{self, DataTableRouteOptions};

let datatable_state = DataTableApiState::new(ctx);

let admin_dt_router = datatable::routes_for_model_with_options(
    "/api/v1/admin/datatable/admin",
    "Admin",
    datatable_state,
    DataTableRouteOptions {
        include_multipart_endpoints: true,
        require_bearer_auth: true, // OpenAPI hint (runtime still enforced by middleware)
    },
)
.route_layer(from_fn_with_state(app_state.clone(), admin_auth_middleware));

let api_router = ApiRouter::new().merge(admin_dt_router);`}</code>
                </pre>

                <h2>Routes Provided By Collection</h2>
                <ul>
                    <li>
                        <code>GET /api/v1/admin/datatable/admin</code> (query)
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin</code> (multipart, optional)
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/json</code>
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/form</code>
                    </li>
                    <li>
                        <code>GET /api/v1/admin/datatable/admin/describe</code>
                    </li>
                    <li>
                        <code>GET|POST /api/v1/admin/datatable/admin/export/stream</code>
                    </li>
                    <li>
                        <code>GET|POST /api/v1/admin/datatable/admin/export/async</code>
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/export/async/json</code>
                    </li>
                    <li>
                        <code>POST /api/v1/admin/datatable/admin/export/async/form</code>
                    </li>
                    <li>
                        <code>GET /api/v1/admin/datatable/admin/export/status?job_id=...</code>
                    </li>
                </ul>

                <h2>OpenAPI Compatibility</h2>
                <ul>
                    <li>
                        <code>/json</code> and <code>/form</code> routes expose typed DTO schema
                        in OpenAPI (<code>BoundDataTableRequestDto</code>, model fixed by route).
                    </li>
                    <li>
                        Query endpoints expose typed query parameters in OpenAPI.
                    </li>
                    <li>
                        Multipart endpoints are still supported for compatibility, but JSON/form are
                        better for API docs and typed clients.
                    </li>
                    <li>
                        Framework adds stable operation IDs for datatable routes (good for client
                        generation).
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`./console route list --json
# check /api/v1/admin/datatable/admin routes and operation_id values`}</code>
                </pre>

                <h2>Request Grammar (Current)</h2>
                <ul>
                    <li>
                        <code>f-&lt;col&gt;</code>, <code>f-like-&lt;col&gt;</code>
                    </li>
                    <li>
                        <code>f-gte-&lt;col&gt;</code>, <code>f-lte-&lt;col&gt;</code>
                    </li>
                    <li>
                        <code>f-date-from-&lt;col&gt;</code>, <code>f-date-to-&lt;col&gt;</code>
                    </li>
                    <li>
                        <code>f-any-&lt;col1|col2|...&gt;</code>,{' '}
                        <code>f-like-any-&lt;col1|col2|...&gt;</code>
                    </li>
                    <li>
                        <code>f-has-&lt;relation&gt;-&lt;col&gt;</code>,{' '}
                        <code>f-has-like-&lt;relation&gt;-&lt;col&gt;</code>
                    </li>
                    <li>
                        <code>f-locale-&lt;col&gt;</code>,{' '}
                        <code>f-locale-like-&lt;col&gt;</code>
                    </li>
                    <li>
                        Nested relation paths use <code>__</code> (example:{' '}
                        <code>f-has-user__profile-display_name</code>).
                    </li>
                </ul>
                <p>
                    Unknown filter behavior uses <code>DATATABLE_UNKNOWN_FILTER_MODE</code> in{' '}
                    <code>.env</code> (<code>ignore|warn|error</code>).
                </p>

                <h2>Advanced Capability Notes</h2>
                <ul>
                    <li>
                        Query scope override per model: use <code>scope</code> hook. This is the
                        right place for default constraints, tenant boundaries, and route-specific
                        base query rules.
                    </li>
                    <li>
                        Custom filters: use <code>filters</code> hook and read custom keys from{' '}
                        <code>input.params</code> (for example <code>f-email-domain</code>).
                    </li>
                    <li>
                        Column transformer: use <code>mappings</code> hook to remove/hide fields
                        and inject computed fields in each output record map.
                    </li>
                    <li>
                        Join and raw SQL: supported via generated query{' '}
                        <code>.unsafe_sql()</code> with typed wrappers{' '}
                        <code>RawClause</code>, <code>RawJoinSpec</code>,{' '}
                        <code>RawSelectExpr</code>, <code>RawOrderExpr</code>.
                    </li>
                    <li>
                        Relation auto-filters (<code>f-has-...</code>,{' '}
                        <code>f-has-like-...</code>) are generated from schema metadata when
                        relation columns are available in the generated adapter.
                    </li>
                    <li>
                        Datatable responses are row-shaped maps. For deeply nested eager payload
                        shapes, prefer a dedicated API endpoint; use datatable for list/filter/
                        export workloads.
                    </li>
                </ul>

                <h2>Admin Curl Examples</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`curl 'http://127.0.0.1:3000/api/v1/admin/datatable/admin?p=1&ipp=30&sorting_column=id&sorting=desc' \\
  -H 'Authorization: Bearer <ACCESS_TOKEN>' \\
  -H 'X-Timezone: +08:00'`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/json \\
  -H 'Authorization: Bearer <ACCESS_TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{
    "page": 1,
    "ipp": 30,
    "sorting_column": "id",
    "sorting": "desc",
    "f-like-name": "alex"
  }'`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`# custom app-level filter keys handled in hooks
curl 'http://127.0.0.1:3000/api/v1/admin/datatable/admin?f-email-domain=example.com&q=alex&with_recent_tokens=1' \\
  -H 'Authorization: Bearer <ACCESS_TOKEN>'`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`# queue async export
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/async/json \\
  -H 'Authorization: Bearer <ACCESS_TOKEN>' \\
  -H 'Content-Type: application/json' \\
  -d '{"sorting_column":"id","sorting":"desc"}'

# check status
curl 'http://127.0.0.1:3000/api/v1/admin/datatable/admin/export/status?job_id=<JOB_ID>' \\
  -H 'Authorization: Bearer <ACCESS_TOKEN>'`}</code>
                </pre>
            </div>
        </div>
    )
}
