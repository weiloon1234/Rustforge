import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter8EndToEndFlow() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Build an End-to-End Flow
                </h1>
                <p className="text-xl text-gray-500">
                    Trace one real feature from schema and generated models through contracts, workflows, admin UI, datatable export, and runtime bootstrap.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Vertical slice used here</h2>
                <p>
                    This recipe uses the scaffold country management flow because it touches multiple framework surfaces at once: framework-owned model data, admin permissions, custom datatable runtime, bootstrap runtime injection, and frontend shared runtime consumers.
                </p>

                <h2>Step 1: Start from the model source of truth</h2>
                <ul>
                    <li>Framework schema owns <code>countries</code>.</li>
                    <li>Primary key is <code>iso2</code>, not a numeric ID.</li>
                    <li>Admin app code consumes the generated country model API rather than inventing a parallel schema or repository shape.</li>
                </ul>
                <p>
                    That gives the vertical slice a stable base before any HTTP or frontend layer exists.
                </p>

                <h2>Step 2: Define the admin permission boundary</h2>
                <p>
                    The feature stays split into read and manage permissions in <code>app/permissions.toml</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[[permissions]]
key = "country.read"
guard = "admin"
label = "Read Countries"
group = "country"

[[permissions]]
key = "country.manage"
guard = "admin"
label = "Manage Countries"
group = "country"`}</code>
                </pre>
                <p>
                    Once generated, the same permission catalog drives backend typed checks, OpenAPI metadata, and frontend typed store helpers.
                </p>

                <h2>Step 3: Define the HTTP contract</h2>
                <p>
                    The admin API uses a normal request DTO for status changes. Keep transport validation in the contract and business rules in the workflow.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCountryStatusUpdateInput {
    #[rf(one_of("enabled", "disabled"))]
    pub status: String,
}`}</code>
                </pre>

                <h2>Step 4: Keep write-side logic in the workflow</h2>
                <p>
                    The workflow should own update semantics and cache invalidation, not the handler.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub async fn update_status(
    state: &AppApiState,
    iso2: &str,
    status: &str,
) -> Result<Country, AppError> {
    let iso2 = normalize_country_iso2(iso2)
        .ok_or_else(|| AppError::NotFound(t(\"Country not found\")))?;
    let status = normalize_country_status(status)
        .ok_or_else(|| AppError::BadRequest(t(\"Invalid country status\")))?;
    let status_enum = GeneratedCountryStatus::from_storage(status)
        .ok_or_else(|| AppError::BadRequest(t(\"Invalid country status\")))?;

    CountryModel::new(DbConn::pool(&state.db), None)
        .update()
        .where_iso2(Op::Eq, iso2.clone())
        .set_status(status_enum)
        .set_updated_at(time::OffsetDateTime::now_utc())
        .save()
        .await?;

    invalidate_bootstrap_country_cache(state).await?;
    // fetch current row and map it to the runtime bootstrap shape
    /* ... */
}`}</code>
                </pre>

                <h2>Step 5: Keep handlers thin</h2>
                <p>
                    The handler should do guard extraction, permission gate, DTO extraction, and workflow handoff. Nothing else.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub async fn update_country_status(
    AuthUser(auth): AuthUser<AdminGuard>,
    Path(iso2): Path<String>,
    ContractJson(req): ContractJson<AdminCountryStatusUpdateInput>,
    State(state): State<AppApiState>,
) -> Result<ApiResponse<AdminCountryStatusUpdateOutput>, AppError> {
    ensure_permissions(&auth, PermissionMode::Any, &[Permission::CountryManage])?;
    let row = workflows::country::update_status(&state, &iso2, &req.status).await?;
    Ok(ApiResponse::ok(AdminCountryStatusUpdateOutput::from(row)))
}`}</code>
                </pre>

                <h2>Step 6: Add the admin datatable</h2>
                <p>
                    Countries use a dedicated datatable contract plus a custom runtime because the source table is framework-owned. The frontend table and CSV export still stay aligned through the same row-to-record pipeline.
                </p>
                <ul>
                    <li>Contract: <code>app/src/contracts/datatable/admin/country.rs</code></li>
                    <li>Runtime hooks: <code>app/src/internal/datatables/v1/admin/country.rs</code></li>
                    <li>Frontend page: <code>frontend/src/admin/pages/other/CountriesPage.tsx</code></li>
                </ul>

                <h2>Step 7: Inject runtime state into bootstrap</h2>
                <p>
                    Enabled countries are also injected into <code>/api/bootstrap.js</code>. That keeps frontend runtime consumers such as <code>ContactInput</code> aligned with the same backend-owned country source.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// bootstrap payload shape
{
  i18n: { /* ... */ },
  countries: Vec<CountryRuntime>,
}`}</code>
                </pre>

                <h2>Step 8: Use the same typed data on the frontend</h2>
                <p>
                    Frontend admin pages should gate actions with typed permission helpers, and shared runtime consumers should read bootstrap-owned country data instead of hardcoded lists.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-tsx">{`const canManage = useAuthStore((s) =>
  useAuthStore.hasPermission(PERMISSION.country_manage, s.account)
)

const countries = availableCountries()
<ContactInput value={value} onChange={setValue} countries={countries} />`}</code>
                </pre>

                <h2>Step 9: Verify the whole slice</h2>
                <ol>
                    <li>Update a country status from the admin page.</li>
                    <li>Refresh <code>/api/bootstrap.js</code> and confirm enabled countries changed.</li>
                    <li>Load any frontend surface using runtime countries and confirm it reflects the same source.</li>
                    <li>Export the countries datatable and confirm the CSV matches the visible column configuration.</li>
                </ol>

                <h2>Verification commands</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p app
make gen-types
npm --prefix frontend run build
curl -H 'Authorization: Bearer <TOKEN>' http://127.0.0.1:3000/api/bootstrap.js
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/country/query \
  -H 'Authorization: Bearer <TOKEN>' \
  -H 'Content-Type: application/json' \
  -d '{"base":{"include_meta":true,"page":1}}'`}</code>
                </pre>

                <h2>What this recipe demonstrates</h2>
                <ul>
                    <li>One permission catalog driving backend and frontend gates.</li>
                    <li>One generated/runtime model surface driving write logic and bootstrap runtime state.</li>
                    <li>One datatable definition controlling both UI and export behavior.</li>
                    <li>One frontend runtime source for shared country-aware components.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/cookbook/add-admin-datatable">Add an Admin DataTable</a> for the focused datatable recipe.</li>
                    <li><a href="#/requests">Requests &amp; Validation</a> for contract and patch semantics.</li>
                    <li><a href="#/permissions">Permissions &amp; AuthZ</a> for matcher and delegation rules.</li>
                    <li><a href="#/feature-autodatatable">AutoDataTable</a> for the lower-level feature behavior.</li>
                </ul>
            </div>
        </div>
    )
}
