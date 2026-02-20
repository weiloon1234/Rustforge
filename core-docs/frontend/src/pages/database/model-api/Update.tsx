import { MethodTable } from './MethodTable'

export function ModelApiUpdate() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxUpdate`</h1>
                <p className="text-xl text-gray-500">
                    Typed update builder for scoped mutation.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <MethodTable
                    rows={[
                        {
                            method: 'set_<field>(value)',
                            returns: 'Self',
                            notes: 'Typed set clauses.',
                        },
                        {
                            method: 'where_<field>(op, value)',
                            returns: 'Self',
                            notes: 'Typed conditions for target rows.',
                        },
                        {
                            method: 'set_meta_<field>(...) / set_meta_<json>_as(&T)',
                            returns: 'Self / Result<Self>',
                            notes: 'Typed meta updates.',
                        },
                        {
                            method: 'clear_attachment_* / delete_attachment_*',
                            returns: 'Self',
                            notes: 'Attachment update helpers.',
                        },
                        {
                            method: 'unsafe_sql()',
                            returns: 'XxxUnsafeUpdate',
                            notes: 'Raw where special-case path (validated builders).',
                        },
                        {
                            method: 'save()',
                            returns: 'Result<u64>',
                            notes: 'Atomic update + side effects in one tx scope.',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let affected = article
    .update()
    .where_status(Op::Eq, ArticleStatus::Draft)
    .set_status(ArticleStatus::Published)
    .set_meta_published_at(time::OffsetDateTime::now_utc())
    .save()
    .await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
