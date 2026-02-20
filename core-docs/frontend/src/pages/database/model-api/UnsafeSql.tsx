export function ModelApiUnsafe() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`unsafe_sql()` Escape Hatch</h1>
                <p className="text-xl text-gray-500">
                    Special-case path for complex SQL while keeping bind safety.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <div className="bg-amber-50 border-l-4 border-amber-500 p-4">
                    <p className="text-sm text-amber-900">
                        <strong>Unsafe means typed-guarantee unsafe</strong>, not “string
                        interpolation allowed.” SQL injection protection still relies on binds.
                    </p>
                </div>

                <p>Access is explicit:</p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`query.unsafe_sql() -> XxxUnsafeQuery
update.unsafe_sql() -> XxxUnsafeUpdate`}</code>
                </pre>

                <p>Raw helper constructors enforce:</p>
                <ul>
                    <li>non-empty SQL</li>
                    <li><code>?</code> placeholders only</li>
                    <li>placeholder count equals bind count</li>
                    <li><code>$1</code>/<code>$n</code> placeholders rejected</li>
                </ul>

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
            </div>
        </div>
    )
}
