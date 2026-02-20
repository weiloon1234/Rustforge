import { MethodTable } from './MethodTable'

export function ModelApiQuery() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxQuery`</h1>
                <p className="text-xl text-gray-500">
                    Typed-first fluent query builder with explicit unsafe escape hatch.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Typed Filters</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'where_<field>(op, value)',
                            returns: 'Self',
                            notes: 'Field-typed filter generated from schema field list.',
                        },
                        {
                            method: 'where_key(id)',
                            returns: 'Self',
                            notes: 'Typed alias for PK equality.',
                        },
                        {
                            method: 'where_key_in(&[ids])',
                            returns: 'Self',
                            notes: 'Typed alias for PK IN list.',
                        },
                        {
                            method: 'where_col(col, op, value)',
                            returns: 'Self',
                            notes: 'Column-enum based filtering.',
                        },
                        {
                            method: 'where_in / where_not_in',
                            returns: 'Self',
                            notes: 'List filters with binds; no raw interpolation.',
                        },
                        {
                            method: 'where_group / or_where_group',
                            returns: 'Self',
                            notes: 'Nested boolean groups.',
                        },
                    ]}
                />

                <h2>Read Operations</h2>
                <MethodTable
                    rows={[
                        { method: 'get()', returns: 'Result<Vec<XxxView>>', notes: 'Fetch list.' },
                        {
                            method: 'first() / first_or_fail()',
                            returns: 'Result<Option<XxxView>> / Result<XxxView>',
                            notes: 'First row helpers.',
                        },
                        {
                            method: 'find(id) / find_or_fail(id)',
                            returns: 'Result<Option<XxxView>> / Result<XxxView>',
                            notes: 'PK helpers.',
                        },
                        {
                            method: 'exists()',
                            returns: 'Result<bool>',
                            notes: 'Existence check.',
                        },
                        {
                            method: 'paginate(page, per_page)',
                            returns: 'Result<Page<XxxView>>',
                            notes: 'Offset pagination.',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let q = article
    .query()
    .where_status(Op::Eq, ArticleStatus::Published)
    .where_key_in(&[1001_i64, 1002_i64])
    .latest();

let first = q.clone().first_or_fail().await?;
let page = q.paginate(1, 20).await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
