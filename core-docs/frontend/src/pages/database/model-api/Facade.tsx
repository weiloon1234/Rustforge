import { MethodTable } from './MethodTable'

export function ModelApiFacade() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`Xxx` Facade</h1>
                <p className="text-xl text-gray-500">
                    Primary entry type generated per schema model (for example <code>Article</code>).
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    The default generated entrypoint is now <code>Xxx</code>, not{' '}
                    <code>XxxModel</code>. This keeps the API easy to remember while retaining
                    strongly typed builders.
                </p>

                <MethodTable
                    rows={[
                        {
                            method: 'new(db, base_url)',
                            returns: 'Xxx',
                            notes: 'Create facade with DbConn and optional base URL.',
                        },
                        {
                            method: 'query()',
                            returns: 'XxxQuery',
                            notes: 'Typed query/read builder.',
                        },
                        {
                            method: 'insert()',
                            returns: 'XxxInsert',
                            notes: 'Typed create builder.',
                        },
                        {
                            method: 'update()',
                            returns: 'XxxUpdate',
                            notes: 'Typed update builder.',
                        },
                        {
                            method: 'find(id)',
                            returns: 'Result<Option<XxxView>>',
                            notes: 'Find by primary key.',
                        },
                        {
                            method: 'delete(id) / restore(id)',
                            returns: 'Result<u64>',
                            notes: 'Delete or restore (restore only when soft-delete enabled).',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use models::generated::models::Article;

let article = Article::new(&state.db, Some(state.cdn_base.clone()));

let one = article.find(1001).await?;
let rows = article.query().latest().limit(20).get().await?;`}</code>
                </pre>

                <p>
                    Keep this facade as your daily entrypoint. Use the submenu pages for builder
                    details.
                </p>
            </div>
        </div>
    )
}
