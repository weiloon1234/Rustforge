import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter11TestingRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 11: Testing Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Test contracts, handlers, and workflows with clear layer boundaries.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Establish a testing strategy that preserves framework DX while catching regressions in contracts and
                    authorization flows.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Admin contracts and handlers are available as baseline test targets.
                    </li>
                    <li>
                        CLI route introspection helps verify permission/guard metadata quickly.
                    </li>
                    <li>
                        Jobs and realtime binaries can be started independently for integration checks.
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add integration tests under <code>app/tests/</code> (not scaffold default; create manually).
                    </li>
                    <li>
                        Add contract validation tests for custom wrapper types and async uniqueness rules.
                    </li>
                    <li>
                        Add end-to-end tests for admin auth + datatable query/export success/failure paths.
                    </li>
                    <li>
                        Add worker-path tests for outbox dispatch and retry handling.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`cargo test -p app
./console route list --json
cargo test -p core-web
cargo test -p core-jobs`}</code>
                </pre>
            </div>
        </div>
    )
}
