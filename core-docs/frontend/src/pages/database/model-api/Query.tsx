import { MethodTable } from './MethodTable'

export function ModelApiQuery() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxQuery`</h1>
                <p className="text-xl text-gray-500">
                    Typed query builder generated from the model-source SSOT, with a narrow raw escape hatch only when the typed surface is not enough.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What `XxxQuery` is for</h2>
                <p>
                    Use <code>XxxQuery</code> for read paths, existence checks, aggregates, scoped mutation targets, and relation-aware fetches. The generated API follows the Rust model source, including the real primary key type. Do not assume every model uses <code>i64</code> IDs.
                </p>

                <h2>Where the surface comes from</h2>
                <ul>
                    <li><strong>Model SSOT:</strong> <code>app/models/*.rs</code> defines fields, enums, relations, soft delete, localized/meta/attachment features, and primary key type.</li>
                    <li><strong>Generator:</strong> db-gen emits field-typed query methods, terminal readers, relation helpers, and datatable adapters.</li>
                    <li><strong>App code:</strong> handlers and workflows consume <code>XxxQuery</code>; they should not reimplement query strings for normal cases.</li>
                </ul>

                <h2>Main query surface</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'where_<field>(op, value)',
                            returns: 'Self',
                            notes: 'Field-typed predicate generated from the model-source field list and field type.',
                        },
                        {
                            method: 'where_key(id) / where_key_in(&[ids])',
                            returns: 'Self',
                            notes: 'Primary-key filters using the model-source PK type, not a hardcoded numeric assumption.',
                        },
                        {
                            method: 'where_col(col, op, value)',
                            returns: 'Self',
                            notes: 'Column-enum based filtering when the field-specific helper is not the right fit.',
                        },
                        {
                            method: 'where_group / or_where_group',
                            returns: 'Self',
                            notes: 'Nested boolean grouping without falling back to string-built SQL.',
                        },
                        {
                            method: 'order_by / latest / oldest / limit / offset',
                            returns: 'Self',
                            notes: 'Typed ordering and pagination-building helpers.',
                        },
                        {
                            method: 'with(Rel::X) / with_scope(Rel::X, ...) / where_has(Rel::X, ...)',
                            returns: 'Self',
                            notes: 'Relation-aware query helpers generated from model-source relations.',
                        },
                    ]}
                />

                <h2>Terminal readers</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'all(db)',
                            returns: 'Result<Vec<XxxRecord>>',
                            notes: 'Normal list reader. Relation preload uses generated relation metadata rather than handwritten joins.',
                        },
                        {
                            method: 'first(db) / first_or_fail(db)',
                            returns: 'Result<Option<XxxRecord>> / Result<XxxRecord>',
                            notes: 'Single-row readers for workflows and handlers.',
                        },
                        {
                            method: 'find(db, id) / find_or_fail(db, id)',
                            returns: 'Result<Option<XxxRecord>> / Result<XxxRecord>',
                            notes: 'Primary-key lookup helpers that respect the actual PK type.',
                        },
                        {
                            method: 'exists(db) / count(db)',
                            returns: 'Result<bool> / Result<i64>',
                            notes: 'Fast terminal checks without dropping to raw SQL.',
                        },
                        {
                            method: 'paginate(db, page, per_page)',
                            returns: 'Result<Page<XxxRecord>>',
                            notes: 'Offset pagination with the same filtering/order pipeline as normal reads.',
                        },
                        {
                            method: 'sum / avg / min_val / max_val',
                            returns: 'Result<Option<T>>',
                            notes: 'Typed aggregate readers for reporting and summary flows.',
                        },
                    ]}
                />

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let q = ArticleModel::query()
    .where_col(ArticleCol::STATUS, Op::Eq, ArticleStatus::Published)
    .where_has(ArticleRel::AUTHOR, |author| {
        author.where_col(UserCol::IS_ACTIVE, Op::Eq, true)
    })
    .latest();

let first = q.clone().first_or_fail(db).await?;
let page = q.clone().paginate(db, 1, 20).await?;
let rows = q
    .with_scope(ArticleRel::COMMENTS, |comments| {
        comments.order_by(CommentCol::CREATED_AT, OrderDir::Desc).limit(5)
    })
    .all(db)
    .await?;`}</code>
                </pre>

                <h2>Customization boundary</h2>
                <p>
                    If the query shape is still normal model work, keep it on <code>XxxQuery</code>. If app code needs computed values, add them on <a href="#/model-api-view"><code>XxxView</code> methods</a> in the model source file. Use <a href="#/model-api-unsafe">Unsafe SQL</a> only when the typed relation, filter, and aggregate surface is genuinely insufficient.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/schema">Model Source Definition</a> for relation and field generation inputs.</li>
                    <li><a href="#/db-gen">Code Generation</a> for generator ownership and manual extension boundaries.</li>
                    <li><a href="#/model-api-relations">Relations &amp; Joins</a> for relation-aware query helpers.</li>
                </ul>
            </div>
        </div>
    )
}
