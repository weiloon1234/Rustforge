export function ModelApiRelations() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Relations &amp; Joins</h1>
                <p className="text-xl text-gray-500">
                    Model-source relations generate the normal join, preload, and relation-filter surface. Raw joins are the exception path.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where relation helpers come from</h2>
                <p>
                    Relation behavior is generated from relation fields in Rust model sources. Use <code>BelongsTo&lt;T&gt;</code>, <code>HasOne&lt;T&gt;</code>, or <code>HasMany&lt;T&gt;</code> plus <code>#[rf(foreign_key = ...)]</code>; that metadata drives typed preload helpers, scoped relation trees, and relation-aware existence filters.
                </p>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub author_id: i64,
    pub country_iso2: String,
    #[rf(foreign_key = "author_id")]
    pub author: BelongsTo<User>,
    #[rf(foreign_key = "article_id")]
    pub comments: HasMany<Comment>,
    #[rf(foreign_key = "article_id")]
    pub cover_image: HasOne<ArticleImage>,
    #[rf(foreign_key = "country_iso2")]
    pub country: BelongsTo<Country>,
}`}</code>
                </pre>

                <h2>What the generator gives you</h2>
                <ul>
                    <li><code>.with(Rel::NAME)</code> and <code>.with_scope(Rel::NAME, ...)</code> preload helpers</li>
                    <li><code>.where_has(Rel::NAME, ...)</code> and <code>.or_where_has(...)</code> existence filters</li>
                    <li>tree-based nested relation loads, counts, and aggregates</li>
                    <li>relation metadata aligned with the real field names and PK/FK types from model source</li>
                </ul>

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let rows = ArticleModel::query()
    .where_has(ArticleRel::COMMENTS, |q| {
        q.where_col(CommentCol::IS_SPAM, Op::Eq, false)
    })
    .with_scope(ArticleRel::COMMENTS, |q| {
        q.order_by(CommentCol::CREATED_AT, OrderDir::Desc)
            .limit(3)
    })
    .with(ArticleRel::AUTHOR)
    .all(db)
    .await?;`}</code>
                </pre>

                <h2>Current framework conventions</h2>
                <ul>
                    <li>Country linkage should use <code>country_iso2</code> and relation metadata should point to <code>countries.iso2</code>.</li>
                    <li>Framework-owned models and app models can participate in the same generated relation surface because generation is layered from framework model sources and app model sources.</li>
                    <li>Use raw join builders only when the relation surface genuinely does not express the query you need.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/schema">Model Source Definition</a> for relation declaration format.</li>
                    <li><a href="#/localized-relations">Localized Relations</a> for locale-aware relation hydration.</li>
                    <li><a href="#/model-api-unsafe">Unsafe SQL</a> for the explicit raw join escape hatch.</li>
                </ul>
            </div>
        </div>
    )
}
