import { MethodTable } from './MethodTable'

export function ModelApiCollections() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Collections</h1>
                <p className="text-xl text-gray-500">
                    Generated collection helpers keep query results as normal <code>Vec&lt;XxxView&gt;</code> while still adding useful typed transforms.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What the collection layer is for</h2>
                <p>
                    Query results stay plain Rust collections. The framework does not wrap them in a heavy custom collection object. Instead, generation adds <code>XxxViewsExt</code> so app code gets a small typed helper surface without giving up normal <code>Vec</code> behavior.
                </p>

                <h2>Main collection helpers</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'ids()',
                            returns: 'Vec<PK>',
                            notes: 'Collect primary key values using the actual model-source PK type.',
                        },
                        {
                            method: 'pluck(f)',
                            returns: 'Vec<R>',
                            notes: 'Project values out of views without inventing a separate collection abstraction.',
                        },
                        {
                            method: 'key_by(f)',
                            returns: 'HashMap<K, XxxView>',
                            notes: 'Index rows by a computed key; the view values are intentionally cloned into the map.',
                        },
                        {
                            method: 'group_by(f)',
                            returns: 'HashMap<K, Vec<XxxView>>',
                            notes: 'Group rows by a computed key while keeping app-facing view values.',
                        },
                    ]}
                />

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use models::generated::models::ArticleViewsExt;

let rows = article.query().latest().get().await?;
let ids = rows.ids();
let by_status = rows.group_by(|v| v.status);
let titles = rows.pluck(|v| v.title.clone());`}</code>
                </pre>

                <h2>Practical rule</h2>
                <p>
                    Keep using normal iterators and normal <code>Vec</code> APIs. Reach for <code>XxxViewsExt</code> only when one of the generated helpers already matches the transform you need.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/model-api-view">`XxxView` &amp; Extensions</a> for the row shape these helpers operate on.</li>
                    <li><a href="#/model-api-query">`XxxQuery`</a> for the read surface that returns these collections.</li>
                    <li><a href="#/model-api">Model API Overview</a> for the complete generated type map.</li>
                </ul>
            </div>
        </div>
    )
}
