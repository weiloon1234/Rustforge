export function ModelApiRelations() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Relations &amp; Joins</h1>
                <p className="text-xl text-gray-500">
                    Schema-declared relations generate the normal join, preload, and relation-filter surface. Raw joins are the exception path.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where relation helpers come from</h2>
                <p>
                    Relation behavior is generated from <code>relations = [...]</code> in schema TOML. That relation metadata drives relation preload helpers, <code>where_has_*</code> filters, and <code>with_*</code> read flows.
                </p>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`relations = [
  "author:belongs_to:User:author_id:id",
  "comments:has_many:Comment:article_id:id",
  "country:belongs_to:Country:country_iso2:iso2"
]`}</code>
                </pre>

                <h2>What the generator gives you</h2>
                <ul>
                    <li><code>with_author()</code>, <code>with_comments()</code>, and related preload helpers</li>
                    <li><code>where_has_comments(...)</code> and similar relation-aware filter helpers</li>
                    <li><code>get_with_relations()</code> app-facing read surfaces</li>
                    <li>relation metadata aligned with the real field names and PK/FK types from schema</li>
                </ul>

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let rows = article
    .query()
    .where_has_comments(|comments| comments.where_is_spam(Op::Eq, false))
    .with_author()
    .get_with_relations()
    .await?;

let users = user
    .query()
    .where_has_country(|country| country.where_status(Op::Eq, CountryStatus::Enabled))
    .get()
    .await?;`}</code>
                </pre>

                <h2>WithRelations and serialization</h2>
                <p>
                    All query methods (<code>.get()</code>, <code>.find()</code>, <code>.first()</code>, etc.) return <code>ModelWithRelations</code>. This struct wraps the inner <code>ModelView</code> with <code>#[serde(flatten)]</code> and implements <code>Deref&lt;Target=ModelView&gt;</code>.
                </p>
                <ul>
                    <li><strong>Rust access</strong>: field access is transparent via <code>Deref</code> — <code>row.username</code> works directly without <code>row.row.username</code>.</li>
                    <li><strong>JSON serialization</strong>: <code>#[serde(flatten)]</code> ensures fields are at the top level — <code>{`{"id": 1, "username": "john"}`}</code>, not <code>{`{"row": {"id": 1, ...}}`}</code>.</li>
                    <li><strong>Unwrap to View</strong>: use <code>.into_row()</code> when you need the plain <code>ModelView</code> (e.g., for API response DTOs).</li>
                    <li><strong>Move fields</strong>: since <code>Deref</code> returns a reference, moving <code>String</code>/<code>Option</code> fields out requires <code>.clone()</code>. Alternatively, call <code>.into_row()</code> first to take ownership.</li>
                </ul>

                <h2>Current framework conventions</h2>
                <ul>
                    <li>Country linkage should use <code>country_iso2</code> and relation metadata should point to <code>countries.iso2</code>.</li>
                    <li>Framework-owned models and app models can participate in the same generated relation surface because generation is layered from framework schemas and app schemas.</li>
                    <li>Use raw join builders only when the relation surface genuinely does not express the query you need.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/schema">Schema Definition</a> for relation declaration format.</li>
                    <li><a href="#/localized-relations">Localized Relations</a> for locale-aware relation hydration.</li>
                    <li><a href="#/model-api-unsafe">Unsafe SQL</a> for the explicit raw join escape hatch.</li>
                </ul>
            </div>
        </div>
    )
}
