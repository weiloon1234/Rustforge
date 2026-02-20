import { MethodTable } from './MethodTable'

export function ModelApiCollections() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Collections (Vec-First)</h1>
                <p className="text-xl text-gray-500">
                    Lightweight collection DX using generated <code>XxxViewsExt</code>.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Query results remain plain <code>Vec&lt;XxxView&gt;</code>. Each model also
                    gets an extension trait for common transforms.
                </p>

                <MethodTable
                    rows={[
                        {
                            method: 'ids()',
                            returns: 'Vec<PK>',
                            notes: 'Collect primary key values from views.',
                        },
                        {
                            method: 'pluck(f)',
                            returns: 'Vec<R>',
                            notes: 'Project values.',
                        },
                        {
                            method: 'key_by(f)',
                            returns: 'HashMap<K, XxxView>',
                            notes: 'Index by key (clones view values intentionally).',
                        },
                        {
                            method: 'group_by(f)',
                            returns: 'HashMap<K, Vec<XxxView>>',
                            notes: 'Group rows by computed key.',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use models::generated::models::ArticleViewsExt;

let rows = article.query().latest().get().await?;
let ids = rows.ids();
let by_status = rows.group_by(|v| v.status);
let titles = rows.pluck(|v| v.title.clone());`}</code>
                </pre>
            </div>
        </div>
    )
}
