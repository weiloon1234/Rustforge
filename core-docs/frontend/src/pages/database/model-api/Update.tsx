import { MethodTable } from './MethodTable'

export function ModelApiUpdate() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxUpdate`</h1>
                <p className="text-xl text-gray-500">
                    Typed scoped mutation builder for normal model updates, feature writes, and safe bulk updates.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What `XxxUpdate` is for</h2>
                <p>
                    Use <code>XxxUpdate</code> to mutate rows selected by typed query-like predicates. It is not just a single-row helper. The generated builder supports scoped bulk updates and keeps feature side effects inside the same save flow.
                </p>

                <h2>Main update surface</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'where_<field>(op, value) / where_key(id)',
                            returns: 'Self',
                            notes: 'Typed target-row selection using the same schema-generated field and PK awareness as queries.',
                        },
                        {
                            method: 'set_<field>(value)',
                            returns: 'Self',
                            notes: 'Typed field assignments for normal column updates.',
                        },
                        {
                            method: 'set_meta_<field>(...) / set_<localized>_lang(...)',
                            returns: 'Self / Result<Self>',
                            notes: 'Feature-aware updates for meta and localized fields.',
                        },
                        {
                            method: 'clear_attachment_* / delete_attachment_* / add_attachment_*',
                            returns: 'Self',
                            notes: 'Attachment mutation helpers that keep attachment side tables in sync.',
                        },
                        {
                            method: 'unsafe_sql()',
                            returns: 'XxxUnsafeUpdate',
                            notes: 'Explicit raw escape hatch for the narrow cases the typed surface does not cover.',
                        },
                        {
                            method: 'save()',
                            returns: 'Result<u64>',
                            notes: 'Applies the scoped update and related side effects in one transaction-aware flow.',
                        },
                    ]}
                />

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let affected = article
    .update()
    .where_status(Op::Eq, ArticleStatus::Draft)
    .set_status(ArticleStatus::Published)
    .set_meta_priority(20)
    .save()
    .await?;`}</code>
                </pre>

                <h2>Important current behavior</h2>
                <ul>
                    <li>Target-row handling follows the schema-defined PK type. Update-side helper flows must not assume numeric IDs.</li>
                    <li>Bulk updates are supported, but app workflows should still own domain rules such as who is allowed to mutate which rows.</li>
                    <li>If update semantics need omitted vs null vs value distinction, keep that in request contracts with <a href="#/requests"><code>Patch&lt;T&gt;</code></a>; <code>XxxUpdate</code> is the model mutation layer, not the HTTP contract layer.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/requests">Requests &amp; Validation</a> for <code>Option&lt;T&gt;</code> vs <code>Patch&lt;T&gt;</code> at the contract boundary.</li>
                    <li><a href="#/model-api-query">`XxxQuery`</a> for the shared filter vocabulary used before mutation.</li>
                    <li><a href="#/model-api-unsafe">Unsafe SQL</a> for the intentionally narrow raw update path.</li>
                </ul>
            </div>
        </div>
    )
}
