export function ModelApiColumns() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxCol` and Typed Filtering</h1>
                <p className="text-xl text-gray-500">
                    Column enum keeps query construction typed and typo-safe.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Every model gets a column enum: <code>XxxCol</code>. Use it for generic
                    filters, projection, ordering, grouping, and list-based predicates.
                </p>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let rows = article
    .query()
    .where_col(ArticleCol::Status, Op::Eq, ArticleStatus::Published)
    .where_in(ArticleCol::Id, &[1001_i64, 1002_i64])
    .order_by(ArticleCol::CreatedAt, OrderDir::Desc)
    .group_by(&[ArticleCol::Status])
    .get()
    .await?;`}</code>
                </pre>

                <p>
                    Keep <code>where_&lt;field&gt;</code> as your first choice. Use{' '}
                    <code>where_col</code>/<code>XxxCol</code> when building dynamic-yet-typed
                    query fragments.
                </p>
            </div>
        </div>
    )
}
