import { MethodTable } from './MethodTable'

export function ModelApiInsert() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxInsert`</h1>
                <p className="text-xl text-gray-500">
                    Typed create builder generated from schema, including feature-aware writes for localized, meta, and attachments.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What `XxxInsert` is for</h2>
                <p>
                    Use <code>XxxInsert</code> when app code is creating model-owned data. It is the generated write surface for normal create flows. It understands the schema-defined field types and optional framework features, so workflows do not need handwritten insert SQL for standard cases.
                </p>

                <h2>Main insert surface</h2>
                <MethodTable
                    rows={[
                        {
                            method: 'set_<field>(value)',
                            returns: 'Self',
                            notes: 'Typed field setters generated from schema fields and enum/value types.',
                        },
                        {
                            method: 'set_<localized>_lang(locale, value)',
                            returns: 'Self',
                            notes: 'Localized field writers when the model enables localized columns.',
                        },
                        {
                            method: 'set_meta_<field>(...) / set_meta_<json>_as(&T)',
                            returns: 'Self / Result<Self>',
                            notes: 'Typed meta writes built from schema meta definitions.',
                        },
                        {
                            method: 'set_attachment_<field>(...) / add_attachment_<field>(...)',
                            returns: 'Self',
                            notes: 'Attachment helpers for single and multi attachment fields.',
                        },
                        {
                            method: 'save()',
                            returns: 'Result<XxxView>',
                            notes: 'Persists the row and related feature tables in one write flow; reuses an existing transaction when present.',
                        },
                    ]}
                />

                <h2>What `save()` owns</h2>
                <ul>
                    <li>Insert the base row using the schema-defined PK strategy and field types.</li>
                    <li>Persist localized/meta/attachment side tables when the model enables those features.</li>
                    <li>Return the app-facing <code>XxxView</code>, not the raw DB row.</li>
                </ul>

                <h2>Usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let view = article
    .insert()
    .set_title("Typed First".to_string())
    .set_status(ArticleStatus::Draft)
    .set_meta_priority(10)
    .set_title_lang(localized::Locale::En, "Typed First")
    .save()
    .await?;`}</code>
                </pre>

                <h2>Customization boundary</h2>
                <p>
                    Keep create-time invariants in workflows and contracts, then call <code>XxxInsert</code> for the actual write. If app code needs custom computed output after save, add it on <a href="#/model-api-view"><code>XxxView</code> extensions</a> rather than forking the generated insert path.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/requests">Requests &amp; Validation</a> for DTO and workflow boundaries before insert.</li>
                    <li><a href="#/model-api-features">Framework Features on Models</a> for localized/meta/attachment write behavior.</li>
                    <li><a href="#/attachments">Attachments</a> for the framework-level attachment runtime surface.</li>
                </ul>
            </div>
        </div>
    )
}
