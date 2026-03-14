const menu = [
    ['#/model-api-facade', 'Xxx (Facade)'],
    ['#/model-api-query', 'XxxQuery'],
    ['#/model-api-insert', 'XxxInsert'],
    ['#/model-api-update', 'XxxUpdate'],
    ['#/model-api-view', 'XxxView & Model Methods'],
    ['#/model-api-columns', 'XxxCol & Filtering'],
    ['#/model-api-relations', 'Relations & Joins'],
    ['#/model-api-unsafe', 'Unsafe SQL'],
    ['#/model-api-collections', 'Collections'],
    ['#/model-api-features', 'Meta / Attachments / Localized'],
]

export function GeneratedApi() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Model API Overview</h1>
                <p className="text-xl text-gray-500">
                    Typed-first generated model API with explicit extension points and a narrow raw escape hatch.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>SSOT ladder</h2>
                <ol>
                    <li>
                        <strong>Model SSOT:</strong> <code>app/models/*.rs</code> defines fields, enums,
                        relations, localized/meta/attachment behavior, and datatable scaffolding.
                    </li>
                    <li>
                        <strong>Generated API:</strong> db-gen emits typed Rust APIs and TS-exportable types.
                    </li>
                    <li>
                        <strong>Manual extension surface:</strong> app code adds <code>XxxView</code> and <code>XxxWithRelations</code>
                        methods in <code>app/models/*.rs</code> via <code>#[rf_record_impl]</code> and <code>#[rf_model_impl]</code>.
                    </li>
                </ol>

                <h2>Daily-use types</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`Xxx
XxxView
XxxQuery
XxxInsert
XxxUpdate
XxxCol
XxxTableAdapter
XxxRel        // when relation helpers exist
XxxViewsExt   // collection helpers`}</code>
                </pre>

                <h2>What stays internal</h2>
                <p>
                    Raw row and unsafe helper internals remain generated, but they are implementation detail
                    surfaces. App code should prefer <code>XxxView</code>, typed query/update builders, and
                    generated model methods.
                </p>
                <ul>
                    <li><code>XxxRow</code>: raw DB shape.</li>
                    <li><code>XxxUnsafeQuery</code> / <code>XxxUnsafeUpdate</code>: explicit raw escape hatches.</li>
                    <li><code>XxxJson</code>: generated JSON projection, not the primary app model.</li>
                </ul>

                <h2>Model API reference</h2>
                <div className="grid grid-cols-1 gap-3 md:grid-cols-2 not-prose">
                    {menu.map(([href, label]) => (
                        <a
                            key={href}
                            href={href}
                            className="rounded-md border border-gray-200 bg-white px-3 py-2 text-sm font-medium text-gray-800 hover:bg-gray-50"
                        >
                            {label}
                        </a>
                    ))}
                </div>

                <h2>Important current behaviors</h2>
                <ul>
                    <li>
                        Enum-backed fields can generate explained values on app-facing outputs such as
                        datatable/view JSON surfaces.
                    </li>
                    <li>
                        PK handling follows the model-source PK type. Do not assume every model uses
                        numeric IDs.
                    </li>
                    <li>
                        View-level extension methods are the intended app customization point for computed
                        values and typed helpers.
                    </li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/schema">Model Source Definition</a> for the generation inputs.
                    </li>
                    <li>
                        <a href="#/db-gen">Code Generation</a> for generator ownership and build flow.
                    </li>
                    <li>
                        <a href="#/requests">Requests &amp; Validation</a> for how DTOs and generated model APIs
                        meet inside handlers/workflows.
                    </li>
                </ul>
            </div>
        </div>
    )
}
