type ModelKeyRow = {
    key: string
    syntax: string
    defaultValue: string
    remarks: string
}

const MODEL_KEYS: ModelKeyRow[] = [
    {
        key: 'table',
        syntax: 'table = "articles"',
        defaultValue: 'snake_case(model key)',
        remarks: 'Database table name.',
    },
    {
        key: 'pk',
        syntax: 'pk = "iso2"',
        defaultValue: 'id',
        remarks: 'Primary key column name.',
    },
    {
        key: 'pk_type',
        syntax: 'pk_type = "String"',
        defaultValue: 'i64',
        remarks: 'Rust primary key type used by generated APIs and relations.',
    },
    {
        key: 'id_strategy',
        syntax: 'id_strategy = "snowflake" | "manual"',
        defaultValue: 'snowflake for i64 id, otherwise manual',
        remarks: 'Strategy follows actual PK type constraints; do not assume every model is snowflake-backed.',
    },
    {
        key: 'fields',
        syntax: 'fields = ["id:i64", "title:string"]',
        defaultValue: '[]',
        remarks: 'Base DB fields.',
    },
    {
        key: 'localized',
        syntax: 'localized = ["title", "summary"]',
        defaultValue: '[]',
        remarks: 'Generates locale-aware field APIs and translation bags.',
    },
    {
        key: 'meta',
        syntax: 'meta = ["seo_title:string", "flags:json"]',
        defaultValue: '[]',
        remarks: 'Generates typed meta readers/writers on app-facing model APIs.',
    },
    {
        key: 'attachment',
        syntax: 'attachment = ["cover:image"]',
        defaultValue: '[]',
        remarks: 'Single attachment slots.',
    },
    {
        key: 'attachments',
        syntax: 'attachments = ["gallery:image"]',
        defaultValue: '[]',
        remarks: 'Multi attachment slots.',
    },
    {
        key: 'relations',
        syntax: 'relations = ["country:belongs_to:Country:country_iso2:iso2"]',
        defaultValue: '[]',
        remarks: 'Relation format: name:kind:target_model:foreign_key:local_key.',
    },
    {
        key: 'touch',
        syntax: 'touch = ["category"]',
        defaultValue: '[]',
        remarks: 'Touch parent updated_at after child writes.',
    },
    {
        key: 'computed',
        syntax: 'computed = ["status_label:String"]',
        defaultValue: '[]',
        remarks: 'Computed app-facing output fields implemented through extension traits.',
    },
    {
        key: 'hidden',
        syntax: 'hidden = ["internal_notes"]',
        defaultValue: '[]',
        remarks: 'Exclude fields from generated JSON projection.',
    },
    {
        key: 'soft_delete',
        syntax: 'soft_delete = true',
        defaultValue: 'false',
        remarks: 'Enable deleted_at-aware query/update behavior.',
    },
]

export function Schema() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Schema Definition</h1>
                <p className="text-xl text-gray-500">
                    TOML schema surface for db-gen, including layered framework/app ownership and typed model behavior.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Schema SSOT</h2>
                <ul>
                    <li>
                        App schema lives in <code>app/schemas/*.toml</code>.
                    </li>
                    <li>
                        Framework-owned models also come from schema, but are layered in by the framework build.
                    </li>
                    <li>
                        Duplicate model or enum names across framework/app layers are a hard error.
                    </li>
                    <li>
                        Generated files are outputs only. Change schema, then regenerate.
                    </li>
                </ul>

                <h2>Enum definitions</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[AdminType]
type = "enum"
storage = "string"
variants = ["Developer", "SuperAdmin", "Admin"]

[PublishState]
type = "enum"
storage = "i16"
variants = [
  { name = "Draft", value = 0 },
  { name = "Published", value = 1 },
  { name = "Archived", value = 2 },
]`}</code>
                </pre>

                <h2>Model key reference</h2>
                <div className="not-prose overflow-x-auto">
                    <table className="min-w-full text-sm border-collapse border border-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="border border-gray-200 px-3 py-2 text-left">Key</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Syntax</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Default</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Remarks</th>
                            </tr>
                        </thead>
                        <tbody>
                            {MODEL_KEYS.map((row) => (
                                <tr key={row.key}>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <code>{row.key}</code>
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <code>{row.syntax}</code>
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">{row.defaultValue}</td>
                                    <td className="border border-gray-200 px-3 py-2">{row.remarks}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <h2>Current important rules</h2>
                <ul>
                    <li>
                        PK behavior follows the schema-defined <code>pk</code> and <code>pk_type</code>; do not hardcode
                        <code>i64</code> assumptions in app code.
                    </li>
                    <li>
                        <code>computed</code> fields are implemented in <code>generated/src/extensions.rs</code>, not in generated files.
                    </li>
                    <li>
                        Country linkage should use <code>country_iso2</code> and the country schema key, not a parallel manual convention.
                    </li>
                    <li>
                        Relation helpers and model APIs are generated from schema. Prefer them over hand-built query conventions.
                    </li>
                </ul>

                <h2>Complete example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[ArticleStatus]
type = "enum"
storage = "string"
variants = ["Draft", "Published", "Archived"]

[model.article_category]
table = "article_categories"
pk = "id"
pk_type = "i64"
id_strategy = "snowflake"
fields = ["id:i64", "status:ArticleStatus"]
localized = ["name"]
soft_delete = true
relations = ["articles:has_many:article:category_id:id"]

[model.article]
table = "articles"
pk = "id"
pk_type = "i64"
id_strategy = "snowflake"
fields = [
  "id:i64",
  "category_id:i64",
  "status:ArticleStatus",
  "slug:string",
  "published_at:datetime",
]
localized = ["title", "summary"]
meta = ["seo_title:string", "reading_minutes:i32", "flags:json"]
attachment = ["cover:image"]
attachments = ["galleries:image"]
relations = ["category:belongs_to:article_category:category_id:id"]
touch = ["category"]
hidden = ["category_id"]
computed = ["status_label:String"]
soft_delete = true`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/db-gen">Code Generation</a> for how schema is turned into APIs.
                    </li>
                    <li>
                        <a href="#/model-api-view">`XxxView` &amp; Extensions</a> for computed field extension rules.
                    </li>
                    <li>
                        <a href="#/feature-localized-relations">Localized &amp; Relationships</a> and{' '}
                        <a href="#/feature-attachments">Attachments</a> for feature-specific schema behavior.
                    </li>
                </ul>
            </div>
        </div>
    )
}
