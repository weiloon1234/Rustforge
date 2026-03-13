import { MethodTable } from './MethodTable'

export function ModelApiFacade() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`Xxx` Facade</h1>
                <p className="text-xl text-gray-500">
                    Primary generated model entrypoint. It is the stable way app code reaches typed query, insert, and update builders.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What the facade is for</h2>
                <p>
                    Every Rust model source gets an <code>Xxx</code> facade. App code should start from this type instead of constructing ad hoc repositories or handwritten SQL helpers for normal model work. The facade carries the DB connection and optional runtime context such as attachment base URL.
                </p>

                <h2>Where it sits in the SSOT ladder</h2>
                <ul>
                    <li><strong>Model-source SSOT:</strong> fields, relations, PK type, and framework features come from Rust model sources in <code>app/models/*.rs</code> plus framework model sources.</li>
                    <li><strong>Generated facade:</strong> <code>Xxx</code> is the generated model entrypoint for those capabilities.</li>
                    <li><strong>Manual app extension:</strong> computed values and view helpers belong on <a href="#/model-api-view"><code>XxxView</code> methods</a>, not on a forked facade.</li>
                </ul>

                <h2>Main facade methods</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'new(db, base_url)',
                            returns: 'Xxx',
                            notes: 'Construct the model facade with a DB handle and optional attachment/CDN base URL.',
                        },
                        {
                            method: 'query()',
                            returns: 'XxxQuery',
                            notes: 'Typed read/query builder generated from model-source fields and relations.',
                        },
                        {
                            method: 'insert()',
                            returns: 'XxxInsert',
                            notes: 'Typed create builder for normal model writes.',
                        },
                        {
                            method: 'update()',
                            returns: 'XxxUpdate',
                            notes: 'Typed scoped mutation builder for normal update flows.',
                        },
                        {
                            method: 'find(id) / find_or_fail(id)',
                            returns: 'Result<Option<XxxView>> / Result<XxxView>',
                            notes: 'Primary-key lookup helpers using the actual model-source PK type.',
                        },
                        {
                            method: 'delete(id) / restore(id)',
                            returns: 'Result<u64>',
                            notes: 'Convenience mutation helpers. Restore exists only for soft-delete models.',
                        },
                    ]}
                />

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use models::generated::models::Article;

let article = Article::new(&state.db, Some(state.cdn_base.clone()));

let one = article.find_or_fail(article_id).await?;
let rows = article.query().latest().limit(20).get().await?;
let created = article.insert().set_title("Hello".to_string()).save().await?;`}</code>
                </pre>

                <h2>Practical rule</h2>
                <p>
                    Treat <code>Xxx</code> as the model boundary and the generated builders as the standard API. Reach for separate repositories only when you are intentionally building a higher-level domain service, not because the generated model facade is supposed to be bypassed by default.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/model-api-query">`XxxQuery`</a> for read/query behavior.</li>
                    <li><a href="#/model-api-insert">`XxxInsert`</a> for create behavior.</li>
                    <li><a href="#/model-api-update">`XxxUpdate`</a> for mutation behavior.</li>
                </ul>
            </div>
        </div>
    )
}
