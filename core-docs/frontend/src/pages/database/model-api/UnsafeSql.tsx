export function ModelApiUnsafe() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Unsafe SQL</h1>
                <p className="text-xl text-gray-500">
                    Explicit raw-query escape hatch for the cases the typed generated model surface does not express. It is intentionally narrow.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <div className="rounded-lg border-l-4 border-amber-500 bg-amber-50 p-4">
                    <p className="text-sm text-amber-900">
                        <strong>Unsafe means generator-guarantee unsafe</strong>, not “free-form interpolation allowed.” Bind safety still matters. The raw API exists because some SQL shapes are genuinely outside the typed query builder, not because handwritten SQL is the default path.
                    </p>
                </div>

                <h2>How you enter the raw path</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`query.unsafe_sql() -> XxxUnsafeQuery
update.unsafe_sql() -> XxxUnsafeUpdate`}</code>
                </pre>

                <h2>What the raw helpers still enforce</h2>
                <ul>
                    <li>non-empty SQL fragments</li>
                    <li><code>?</code> placeholders only</li>
                    <li>placeholder count must match bind count</li>
                    <li><code>$1</code> / numbered-postgres placeholder style is rejected in these helper constructors</li>
                    <li>bind values still go through explicit raw helper types instead of string interpolation</li>
                </ul>

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_db::common::sql::{
    RawClause, RawJoinSpec, RawOrderExpr, RawSelectExpr,
};

let where_clause = RawClause::new("a.views > ? AND a.deleted_at IS NULL", [100])?;
let join_on = RawClause::new("u.id = a.author_id", Vec::<i32>::new())?;
let join = RawJoinSpec::left("users u", join_on)?;

let rows = article
    .query()
    .unsafe_sql()
    .where_raw(where_clause)
    .join_raw(join)
    .add_select_raw(RawSelectExpr::new("u.name AS author_name")?)
    .order_by_raw(RawOrderExpr::new("a.created_at DESC NULLS LAST")?)
    .done()
    .get()
    .await?;`}</code>
                </pre>

                <h2>When to use it</h2>
                <ul>
                    <li>cross-table SQL shapes that exceed generated relation helpers</li>
                    <li>database-specific expressions not represented by the typed query API</li>
                    <li>rare reporting or maintenance cases where the typed surface would become more awkward than the query itself</li>
                </ul>

                <h2>When not to use it</h2>
                <ul>
                    <li>normal field filtering, ordering, grouping, and aggregates</li>
                    <li>normal relation preloads and <code>where_has_*</code> flows</li>
                    <li>standard insert/update behavior already covered by generated builders</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/model-api-query">`XxxQuery`</a> for the normal typed read surface.</li>
                    <li><a href="#/model-api-relations">Relations &amp; Joins</a> for generated relation helpers.</li>
                    <li><a href="#/db-gen">Code Generation</a> for where the raw helper types come from.</li>
                </ul>
            </div>
        </div>
    )
}
