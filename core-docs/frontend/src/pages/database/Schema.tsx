type ModelKeyRow = {
    key: string
    syntax: string
    defaultValue: string
    remarks: string
}

type TypeRow = {
    token: string
    details: string
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
        syntax: 'pk = "id"',
        defaultValue: 'id',
        remarks: 'Primary key column name.',
    },
    {
        key: 'pk_type',
        syntax: 'pk_type = "i64"',
        defaultValue: 'i64',
        remarks: 'Primary key Rust type used by generated APIs.',
    },
    {
        key: 'id_strategy',
        syntax: 'id_strategy = "snowflake" | "manual"',
        defaultValue: 'snowflake (for i64), otherwise manual',
        remarks: 'snowflake currently requires pk_type = i64.',
    },
    {
        key: 'fields',
        syntax: 'fields = ["id:i64", "title:string"]',
        defaultValue: '[]',
        remarks: 'Base DB fields in name:type format.',
    },
    {
        key: 'multilang',
        syntax: 'multilang = ["title", "summary"]',
        defaultValue: '[]',
        remarks: 'Generates locale-aware field getters/setters and translation maps.',
    },
    {
        key: 'meta',
        syntax: 'meta = ["seo_title:string", "flags:json"]',
        defaultValue: '[]',
        remarks: 'Typed meta helpers on generated view/insert/update APIs.',
    },
    {
        key: 'attachment',
        syntax: 'attachment = ["cover:image"]',
        defaultValue: '[]',
        remarks: 'Single attachment fields.',
    },
    {
        key: 'attachments',
        syntax: 'attachments = ["galleries:image"]',
        defaultValue: '[]',
        remarks: 'Multiple attachment fields.',
    },
    {
        key: 'relations',
        syntax: 'relations = ["category:belongs_to:article_category:category_id:id"]',
        defaultValue: '[]',
        remarks: 'Relation format: name:kind:target_model:foreign_key:local_key.',
    },
    {
        key: 'touch',
        syntax: 'touch = ["category"]',
        defaultValue: '[]',
        remarks: 'Auto-updates belongs_to parent updated_at after child writes.',
    },
    {
        key: 'computed',
        syntax: 'computed = ["display_status:String"]',
        defaultValue: '[]',
        remarks: 'Adds computed fields to JSON output (implemented via extension trait).',
    },
    {
        key: 'hidden',
        syntax: 'hidden = ["internal_notes", "meta"]',
        defaultValue: '[]',
        remarks: 'Excludes fields from generated JSON shape.',
    },
    {
        key: 'soft_delete',
        syntax: 'soft_delete = true',
        defaultValue: 'false',
        remarks: 'Auto-adds deleted_at: Option<datetime> if not already present.',
    },
    {
        key: 'disable_id',
        syntax: 'disable_id = true',
        defaultValue: 'false',
        remarks: 'Disables auto id field insertion.',
    },
    {
        key: 'disable_timestamps',
        syntax: 'disable_timestamps = true',
        defaultValue: 'false',
        remarks: 'Disables auto created_at/updated_at insertion.',
    },
]

const FIELD_TYPES: TypeRow[] = [
    { token: 'string', details: 'Mapped to Rust String.' },
    { token: 'datetime', details: 'Mapped to time::OffsetDateTime (RFC3339 serde).' },
    { token: 'uuid', details: 'Mapped to uuid::Uuid.' },
    { token: 'hashed', details: 'Stored as String; generated API treats it as hashed field type.' },
    { token: 'i16 / i32 / i64 / f64 / bool ...', details: 'Standard Rust scalar types are supported.' },
    {
        token: 'Enum type (e.g. ArticleStatus)',
        details: 'Use a top-level enum section in schema and reference it in fields.',
    },
]

const META_TYPES: TypeRow[] = [
    { token: 'string', details: 'Typed accessor/setter as String.' },
    { token: 'bool', details: 'Typed accessor/setter as bool.' },
    { token: 'i32 / i64 / f64', details: 'Typed numeric accessors/setters.' },
    { token: 'json', details: 'JSON value with *_as<T>() helper support.' },
    { token: 'datetime', details: 'Stored as RFC3339 string and parsed to OffsetDateTime accessor.' },
    {
        token: 'Custom Type (e.g. ExtraMeta or my_mod::ExtraMeta)',
        details: 'Custom typed serializer/deserializer helpers are generated.',
    },
]

export function Schema() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Schema Definition</h1>
                <p className="text-xl text-gray-500">
                    Complete TOML schema surface for <code>db-gen</code> in{' '}
                    <code>app/schemas/*.toml</code>.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Single Source of Truth</h2>
                <ul>
                    <li>
                        Define schema in <code>app/schemas/*.toml</code>.
                    </li>
                    <li>
                        Generated model APIs are written to <code>generated/src</code>.
                    </li>
                    <li>
                        Never manually edit generated model files; change schema and regenerate.
                    </li>
                    <li>
                        Schema files are merged by directory. Duplicate model/enum names will fail
                        generation.
                    </li>
                </ul>

                <h2>Top-Level Enum Definitions</h2>
                <p>
                    Enums are defined as top-level sections. They can be string-backed or integer-backed.
                </p>
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
                <p>
                    <strong>Remark:</strong> integer enum storage requires explicit variant values.
                </p>

                <h2>Model Key Reference</h2>
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
                                    <td className="border border-gray-200 px-3 py-2">
                                        {row.defaultValue}
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">{row.remarks}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <h2>Field Type Tokens</h2>
                <div className="not-prose overflow-x-auto">
                    <table className="min-w-full text-sm border-collapse border border-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="border border-gray-200 px-3 py-2 text-left">Type Token</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Behavior</th>
                            </tr>
                        </thead>
                        <tbody>
                            {FIELD_TYPES.map((row) => (
                                <tr key={row.token}>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <code>{row.token}</code>
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">{row.details}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <h2>Meta Type Tokens</h2>
                <div className="not-prose overflow-x-auto">
                    <table className="min-w-full text-sm border-collapse border border-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="border border-gray-200 px-3 py-2 text-left">Meta Type</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Behavior</th>
                            </tr>
                        </thead>
                        <tbody>
                            {META_TYPES.map((row) => (
                                <tr key={row.token}>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <code>{row.token}</code>
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">{row.details}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <h2>Attachment Type Source</h2>
                <p>
                    Attachment field type names (for example <code>image</code>) must exist in{' '}
                    <code>app/configs.toml</code> under <code>[attachment_type.*]</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[attachment_type.image]
allowed = ["image/jpeg", "image/png", "image/webp"]

[attachment_type.image.resize]
width = 1600
height = 900
quality = 85`}</code>
                </pre>

                <h2>Complete Model Example (All Common Features)</h2>
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
multilang = ["name"]
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
multilang = ["title", "summary"]
meta = ["seo_title:string", "reading_minutes:i32", "flags:json"]
attachment = ["cover:image"]
attachments = ["galleries:image"]
relations = ["category:belongs_to:article_category:category_id:id"]
touch = ["category"]
hidden = ["category_id"]
# computed fields require extension trait implementation
computed = ["status_label:String"]
soft_delete = true`}</code>
                </pre>

                <h2>Computed Field Remark</h2>
                <p>
                    If you define <code>computed</code>, implement the generated extension trait in{' '}
                    <code>generated/src/extensions.rs</code> (for example{' '}
                    <code>ArticleComputed</code> for <code>ArticleView</code>), so computed values
                    can be emitted in JSON.
                </p>

                <h2>Operational Notes</h2>
                <ul>
                    <li>
                        Schema and SQL migrations are separate responsibilities. Define tables and
                        constraints in SQL migrations.
                    </li>
                    <li>
                        Keep enum storage and DB column type aligned (text vs i16/i32/i64).
                    </li>
                    <li>
                        <code>touch</code> should target relations whose parent has{' '}
                        <code>updated_at</code>.
                    </li>
                    <li>
                        Regenerate with <code>cargo check -p generated</code> after schema changes.
                    </li>
                </ul>
            </div>
        </div>
    )
}
