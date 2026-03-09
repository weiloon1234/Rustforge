import { useEffect } from 'react'
import Prism from 'prismjs'

export function AddAdminDatatable() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Add an Admin DataTable
                </h1>
                <p className="text-xl text-gray-500">
                    Build a scoped admin datatable with one contract SSOT, typed row hooks, and query/export parity.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Add a new admin datatable without inventing per-table request DTOs or splitting display/export logic into separate pipelines.
                </p>

                <h2>What stays single-source-of-truth</h2>
                <ul>
                    <li>
                        Datatable contract: <code>app/src/contracts/datatable/admin/&lt;resource&gt;.rs</code>
                    </li>
                    <li>
                        Runtime hooks: <code>app/src/internal/datatables/v1/admin/&lt;resource&gt;.rs</code>
                    </li>
                    <li>
                        Route registration: <code>app/src/internal/datatables/v1/admin/mod.rs</code>
                    </li>
                    <li>
                        Frontend visible/exported columns: the page-level <code>columns</code> definition in the React datatable page
                    </li>
                </ul>

                <h2>Step 1: Define the datatable contract</h2>
                <p>
                    The contract owns the scoped key, route prefix, row DTO, and filter metadata. Do not create custom request DTOs for normal datatables.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/src/contracts/datatable/admin/content_page.rs
pub const SCOPED_KEY: &str = "admin.content_page";
pub const ROUTE_PREFIX: &str = "/datatable/content_page";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ContentPageDatatableRow {
    pub id: SnowflakeId,
    pub tag: String,
    pub title: Option<String>,
    pub is_system: ContentPageSystemFlag,
    pub is_system_explained: String,
    pub updated_at: String,
}

impl DataTableScopedContract for AdminContentPageDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = ContentPageDatatableRow;

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        // contract owns filter UI metadata
        vec![/* ... */]
    }
}`}</code>
                </pre>

                <h2>Step 2: Implement typed hooks</h2>
                <p>
                    Put row-level security and custom filters in the datatable hooks file. Keep display/export shaping on the typed row with <code>map_row</code> and <code>row_to_record</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`impl ContentPageDataTableHooks for ContentPageDataTableAppHooks {
    fn authorize(&self, _input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<()> {
        // permission gate here
        Ok(())
    }

    fn filter_query<'db>(
        &'db self,
        query: ContentPageQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<ContentPageQuery<'db>>> {
        match filter_key {
            "q" => Ok(Some(query.where_group(|q| {
                q.where_col(ContentPageCol::Tag, Op::Like, format!("%{value}%"))
                 .or_where_col(ContentPageCol::Title, Op::Like, format!("%{value}%"))
            }))),
            _ => Ok(None),
        }
    }

    fn map_row(&self, row: &mut ContentPageWithRelations, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<()> {
        row.identity = row.identity();
        Ok(())
    }

    fn row_to_record(
        &self,
        row: ContentPageWithRelations,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        self.default_row_to_record(row)
    }
}`}</code>
                </pre>

                <h2>Step 3: Register once</h2>
                <p>
                    Add the datatable to the admin datatable catalog and mount routes from the same admin datatable module. Do not create ad hoc route trees for each table.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub static ADMIN_SCOPED_DATATABLES: &[ScopedDatatableSpec] = &[
    ScopedDatatableSpec {
        scoped_key: "admin.content_page",
        route_prefix: "/datatable/content_page",
        register: content_page::register_scoped,
        mount_routes: content_page::routes,
    },
];`}</code>
                </pre>

                <h2>Step 4: Keep frontend columns as export SSOT</h2>
                <p>
                    The React datatable now sends the visible export column map to the backend. That means the page-level column definition controls visible order, labels, and alternate export fields.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-tsx">{`<DataTable<ContentPageDatatableRow>
  url="/api/v1/admin/datatable/content_page/query"
  exportCsvUrl="/api/v1/admin/datatable/content_page/export/csv"
  columns={[
    { key: 'actions', label: 'Actions', exportIgnore: true, render: () => /* ... */ },
    { key: 'tag', label: 'Tag' },
    { key: 'title', label: 'Title' },
    {
      key: 'is_system',
      label: 'System',
      exportColumn: 'is_system_explained',
      render: (row) => row.is_system_explained,
    },
    { key: 'updated_at', label: 'Updated At' },
  ]}
  exportIgnoreColumns={['id']}
/>`}</code>
                </pre>
                <p>
                    Keep value transformations on the typed backend row, then choose which field the CSV should emit with <code>exportColumn</code>. Do not fork separate display-only and export-only mapping pipelines.
                </p>

                <h2>Step 5: Add page-level summary only when needed</h2>
                <p>
                    Per-page footer metrics and cross-page totals are different concerns. Footer metrics can use the current page records; cross-page totals need a backend summary endpoint that reuses the same filters.
                </p>

                <h2>Verification</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p app
make gen-types
curl -X POST http://127.0.0.1:3000/api/v1/admin/datatable/content_page/query \
  -H 'Authorization: Bearer <TOKEN>' \
  -H 'Content-Type: application/json' \
  -d '{"base":{"include_meta":true,"page":1},"q":"privacy"}'`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/feature-autodatatable">AutoDataTable</a> for the feature-level runtime and route behavior.</li>
                    <li><a href="#/permissions">Permissions &amp; AuthZ</a> for query/export permission rules.</li>
                    <li><a href="#/cookbook/build-end-to-end-flow">Build an End-to-End Flow</a> for a full vertical slice using the same pieces together.</li>
                </ul>
            </div>
        </div>
    )
}
