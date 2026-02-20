import { MethodTable } from './MethodTable'

export function ModelApiInsert() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxInsert`</h1>
                <p className="text-xl text-gray-500">
                    Typed create builder with atomic save behavior.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <MethodTable
                    rows={[
                        {
                            method: 'set_<field>(value)',
                            returns: 'Self',
                            notes: 'Typed setters generated from DB fields.',
                        },
                        {
                            method: 'set_<localized>_lang(locale, value)',
                            returns: 'Self',
                            notes: 'Localized field writes.',
                        },
                        {
                            method: 'set_meta_<field>(...) / set_meta_<json>_as(&T)',
                            returns: 'Self / Result<Self>',
                            notes: 'Typed meta writes.',
                        },
                        {
                            method: 'set_attachment_<field>(...)',
                            returns: 'Self',
                            notes: 'Attachment writes.',
                        },
                        {
                            method: 'save()',
                            returns: 'Result<XxxView>',
                            notes: 'Auto-atomic for pool-backed DbConn; reuses tx if already in tx.',
                        },
                    ]}
                />

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let view = article
    .insert()
    .set_title("Typed First".to_string())
    .set_status(ArticleStatus::Draft)
    .set_meta_priority(10)
    .save()
    .await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
