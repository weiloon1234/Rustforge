export function DbGen() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Code Generation (db-gen)</h1>
                <p className="text-xl text-gray-500">
                    Build-time typed API generation from <code>app/schemas/*.toml</code>.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    <code>generated/build.rs</code> runs db-gen and writes generated model/guard
                    code into <code>generated/src</code>.
                </p>
                <p>
                    Datatable hooks are scaffolded into <code>app/src/internal/datatables</code>.
                    Generator keeps <code>mod.generated.rs</code> in sync and creates per-model
                    hook files once.
                </p>

                <h2>Generation Commands</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo check --workspace
cargo check -p generated`}</code>
                </pre>

                <h2>Generated Surface</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`Article
ArticleQuery
ArticleInsert
ArticleUpdate
ArticleView
ArticleDataTable
ArticleDataTableConfig
ArticleDataTableHooks
ArticleViewsExt`}</code>
                </pre>
            </div>
        </div>
    )
}
