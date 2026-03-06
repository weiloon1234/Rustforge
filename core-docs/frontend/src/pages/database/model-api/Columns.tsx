import { MethodTable } from './MethodTable'

export function ModelApiColumns() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxCol` &amp; Typed Filtering</h1>
                <p className="text-xl text-gray-500">
                    Column enums are the generic typed surface for dynamic query construction when field-specific helpers are not enough.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What `XxxCol` is for</h2>
                <p>
                    Every generated model gets an <code>XxxCol</code> enum. Use it when app code needs dynamic-yet-typed filtering, ordering, grouping, or projection. For fixed queries, field-specific helpers like <code>where_status(...)</code> are still the better first choice.
                </p>

                <h2>Main usage points</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'where_col(col, op, value)',
                            returns: 'Self',
                            notes: 'Generic typed predicate when the field name is chosen dynamically.',
                        },
                        {
                            method: 'where_in / where_not_in',
                            returns: 'Self',
                            notes: 'Column-enum list filtering with binds, not interpolated raw SQL.',
                        },
                        {
                            method: 'order_by / order_by_nulls_first / order_by_nulls_last',
                            returns: 'Self',
                            notes: 'Typed ordering without handwritten column strings.',
                        },
                        {
                            method: 'group_by / group_by_raw',
                            returns: 'Self',
                            notes: 'Use the enum-based path first; raw grouping is the explicit fallback.',
                        },
                        {
                            method: 'select_cols / add_select_cols',
                            returns: 'Self',
                            notes: 'Generic projection helpers when app code is building reusable query fragments.',
                        },
                    ]}
                />

                <h2>Usage example</h2>
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

                <h2>Practical rule</h2>
                <p>
                    Use field-specific methods when the query shape is known at compile time. Use <code>XxxCol</code> when app code is assembling generic query logic and still wants generator-backed type safety.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/model-api-query">`XxxQuery`</a> for the full query-builder surface.</li>
                    <li><a href="#/schema">Schema Definition</a> for the field list that generates the enum.</li>
                    <li><a href="#/model-api-unsafe">Unsafe SQL</a> for the explicit fallback when enum-based composition is still not enough.</li>
                </ul>
            </div>
        </div>
    )
}
