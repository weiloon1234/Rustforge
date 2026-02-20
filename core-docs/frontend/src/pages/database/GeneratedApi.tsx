export function GeneratedApi() {
    const menu = [
        ['#/model-api-facade', 'Xxx (Facade)'],
        ['#/model-api-query', 'XxxQuery'],
        ['#/model-api-insert', 'XxxInsert'],
        ['#/model-api-update', 'XxxUpdate'],
        ['#/model-api-view', 'XxxView'],
        ['#/model-api-columns', 'XxxCol / Filtering'],
        ['#/model-api-relations', 'Relations / Joins'],
        ['#/model-api-unsafe', 'Unsafe SQL'],
        ['#/model-api-collections', 'Collections'],
        ['#/model-api-features', 'Meta / Attachments / Localized'],
    ]

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Model API Overview</h1>
                <p className="text-xl text-gray-500">
                    Typed-first generated API with explicit raw special-case path.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Typed-First Ladder</h2>
                <ol>
                    <li>
                        <strong>Level 1 (default):</strong> typed fluent API (
                        <code>where_&lt;field&gt;</code>, <code>where_in</code>, typed enums,
                        typed relation helpers).
                    </li>
                    <li>
                        <strong>Level 2:</strong> advanced typed composition (
                        <code>where_group</code>, relation predicates, aggregates, pagination).
                    </li>
                    <li>
                        <strong>Level 3 (special case):</strong> explicit{' '}
                        <code>unsafe_sql()</code> wrappers with validated raw helper types.
                    </li>
                </ol>

                <h2>Public Daily-Use Types</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`Xxx
XxxView
XxxQuery
XxxInsert
XxxUpdate
XxxCol
XxxTableAdapter
XxxRel          // only when has_many relations exist
XxxViewsExt     // Vec<XxxView> collection helpers`}</code>
                </pre>

                <p>
                    Internal support types remain generated for implementation boundaries and are
                    marked as hidden in docs (for example <code>XxxRow</code>,{' '}
                    <code>XxxUnsafeQuery</code>, <code>XxxUnsafeUpdate</code>).
                </p>

                <h2>Reference Menu</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 not-prose">
                    {menu.map(([href, label]) => (
                        <a
                            key={href}
                            href={href}
                            className="px-3 py-2 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                        >
                            {label}
                        </a>
                    ))}
                </div>

                <h2>Unsafe SQL Naming Clarification</h2>
                <div className="bg-amber-50 border-l-4 border-amber-500 p-4">
                    <p className="text-sm text-amber-900">
                        <strong>`unsafe_sql()` means typed-guarantee unsafe</strong>, not SQL
                        injection unsafe. Binds are still required and validated via raw helper
                        constructors.
                    </p>
                </div>
            </div>
        </div>
    )
}
