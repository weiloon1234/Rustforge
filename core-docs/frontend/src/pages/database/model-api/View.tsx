import { MethodTable } from './MethodTable'

export function ModelApiView() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxView`</h1>
                <p className="text-xl text-gray-500">
                    Hydrated read model returned by query/find/save flows.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    <code>XxxView</code> contains DB fields plus framework enrichments:
                    localized values, meta bag, and attachments URLs.
                </p>

                <MethodTable
                    rows={[
                        {
                            method: 'update(db)',
                            returns: 'XxxUpdate',
                            notes: 'Start update scoped to this row primary key.',
                        },
                        {
                            method: 'update_with(&Xxx)',
                            returns: 'XxxUpdate',
                            notes: 'Use existing facade context.',
                        },
                        {
                            method: 'to_json()',
                            returns: 'XxxJson',
                            notes: 'Projection respecting hidden/computed settings.',
                        },
                        {
                            method: 'meta_<field>()',
                            returns: 'Option<T> or Result<Option<T>>',
                            notes: 'Typed meta readers.',
                        },
                        {
                            method: 'meta_<json>_as<U>()',
                            returns: 'Result<Option<U>>',
                            notes: 'Fallback for dynamic JSON-shaped fields.',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let view = article.query().find_or_fail(1001).await?;

let featured = view.meta_is_featured().unwrap_or(false);
let extra = view.meta_extra()?; // for meta = ["extra:ExtraMeta"]`}</code>
                </pre>
            </div>
        </div>
    )
}
