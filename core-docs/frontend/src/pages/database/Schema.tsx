type ModelKeyRow = {
    key: string
    syntax: string
    defaultValue: string
    remarks: string
}

const MODEL_KEYS: ModelKeyRow[] = [
    {
        key: '#[rf_model(table = ...)]',
        syntax: '#[rf_model(table = "articles", soft_delete)]',
        defaultValue: 'table = snake_case(struct name)',
        remarks: 'Model-level table name and flags such as soft delete.',
    },
    {
        key: '#[rf(pk(strategy = ...))]',
        syntax: '#[rf(pk(strategy = "snowflake"))] pub id: i64',
        defaultValue: 'field named id',
        remarks: 'Primary key marker and optional ID strategy.',
    },
    {
        key: '#[rf_db_enum(storage = ...)]',
        syntax: '#[rf_db_enum(storage = "string")]',
        defaultValue: 'n/a',
        remarks: 'Enum storage format for generated DB/API helpers.',
    },
    {
        key: 'Localized<T>',
        syntax: 'pub title: Localized<String>',
        defaultValue: 'n/a',
        remarks: 'Locale-aware field with generated translation helpers.',
    },
    {
        key: 'Meta<T>',
        syntax: 'pub seo_title: Meta<String>',
        defaultValue: 'n/a',
        remarks: 'Typed meta readers and writers on generated APIs.',
    },
    {
        key: 'Attachment / Attachments',
        syntax: '#[rf(kind = "image")] pub cover: Attachment',
        defaultValue: 'n/a',
        remarks: 'Single or multiple attachment slots tied to config attachment types.',
    },
    {
        key: 'BelongsTo<T> / HasMany<T>',
        syntax: '#[rf(foreign_key = "author_id")] pub author: BelongsTo<User>',
        defaultValue: 'n/a',
        remarks: 'Typed relation metadata for query helpers and relation loading.',
    },
    {
        key: '#[rf(hidden)]',
        syntax: '#[rf(hidden)] pub internal_notes: Option<String>',
        defaultValue: 'visible',
        remarks: 'Exclude a field from generated JSON projections.',
    },
    {
        key: '#[rf_view_impl]',
        syntax: '#[rf_view_impl] impl ArticleView { ... }',
        defaultValue: 'n/a',
        remarks: 'Adds plain generated methods directly onto XxxView.',
    },
    {
        key: '#[rf_computed]',
        syntax: '#[rf_computed] pub fn status_label(&self) -> String',
        defaultValue: 'plain method only',
        remarks: 'Also exports the method value into generated JSON projections.',
    },
    {
        key: '#[rf_with_relations_impl]',
        syntax: '#[rf_with_relations_impl] impl ArticleWithRelations { ... }',
        defaultValue: 'n/a',
        remarks: 'Adds methods on relation-loaded read models.',
    },
]

export function Schema() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Model Source Definition</h1>
                <p className="text-xl text-gray-500">
                    Rust model-source surface for db-gen, including layered framework/app ownership and typed model behavior.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Model-Source SSOT</h2>
                <ul>
                    <li>
                        App model source lives in <code>app/models/*.rs</code>.
                    </li>
                    <li>
                        Framework-owned models also come from Rust model sources, layered in by the framework build.
                    </li>
                    <li>
                        Duplicate model or enum names across framework/app layers are a hard error.
                    </li>
                    <li>
                        Generated files are outputs only. Change model source, then regenerate.
                    </li>
                </ul>

                <h2>Enum definitions</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rf_db_enum(storage = "string")]
pub enum AdminType {
    Developer,
    SuperAdmin,
    Admin,
}

#[rf_db_enum(storage = "i16")]
pub enum PublishState {
    Draft = 0,
    Published = 1,
    Archived = 2,
}`}</code>
                </pre>

                <h2>Model source reference</h2>
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
                        PK behavior follows the model field type and optional <code>#[rf(pk(...))]</code>; do not hardcode
                        <code>i64</code> assumptions in app code.
                    </li>
                    <li>
                        <code>#[rf_computed]</code> methods live in <code>#[rf_view_impl]</code> blocks inside <code>app/models/*.rs</code>.
                    </li>
                    <li>
                        Country linkage should use <code>country_iso2</code> and the country model field, not a parallel manual convention.
                    </li>
                    <li>
                        Relation helpers and model APIs are generated from model source. Prefer them over hand-built query conventions.
                    </li>
                </ul>

                <h2>Complete example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rf_db_enum(storage = "string")]
pub enum ArticleStatus {
    Draft,
    Published,
    Archived,
}

#[rf_model(table = "article_categories", soft_delete)]
pub struct ArticleCategory {
    pub id: i64,
    pub status: ArticleStatus,
    pub name: Localized<String>,
    #[rf(foreign_key = "category_id")]
    pub articles: HasMany<Article>,
}

#[rf_model(table = "articles", soft_delete)]
pub struct Article {
    pub id: i64,
    pub category_id: i64,
    pub status: ArticleStatus,
    pub slug: String,
    pub published_at: Option<time::OffsetDateTime>,
    pub title: Localized<String>,
    pub summary: Localized<String>,
    pub seo_title: Meta<String>,
    pub reading_minutes: Meta<i32>,
    pub flags: Meta<serde_json::Value>,
    #[rf(kind = "image")]
    pub cover: Attachment,
    #[rf(kind = "image")]
    pub galleries: Attachments,
    #[rf(hidden)]
    pub internal_notes: Option<String>,
    #[rf(foreign_key = "category_id", touch)]
    pub category: BelongsTo<ArticleCategory>,
}

#[rf_view_impl]
impl ArticleView {
    #[rf_computed]
    pub fn status_label(&self) -> String {
        self.status.explained_label()
    }
}`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/db-gen">Code Generation</a> for how model sources are turned into APIs.
                    </li>
                    <li>
                        <a href="#/model-api-view">`XxxView` &amp; model methods</a> for computed field rules.
                    </li>
                    <li>
                        <a href="#/feature-localized-relations">Localized &amp; Relationships</a> and{' '}
                        <a href="#/feature-attachments">Attachments</a> for feature-specific model-source behavior.
                    </li>
                </ul>
            </div>
        </div>
    )
}
