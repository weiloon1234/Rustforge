import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter10CachingRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 10: Caching Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Use cache as a read-optimization layer while preserving source-of-truth in DB/workflow logic.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Add predictable cache keys and invalidation points without coupling cache logic to transport code.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        API state baseline: <code>app/src/internal/api/state.rs</code>
                    </li>
                    <li>
                        Workflows are the right place to perform write-side invalidation.
                    </li>
                    <li>
                        DTO contracts remain the HTTP SSOT; cache never replaces contract validation.
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add cache client to <code>AppApiState</code> and expose helper methods for domain keys.
                    </li>
                    <li>Cache read-heavy datatable metadata and expensive aggregate endpoints.</li>
                    <li>
                        Invalidate on workflow write paths only; avoid invalidation in multiple handler branches.
                    </li>
                    <li>
                        Concept endpoint examples (article/admin dashboards) should live under{' '}
                        <code>app/src/internal/api/v1/&lt;portal&gt;/</code>.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# cold request should populate cache
# repeated request should hit cache
# write action should invalidate and force refresh on next read`}</code>
                </pre>
            </div>
        </div>
    )
}
