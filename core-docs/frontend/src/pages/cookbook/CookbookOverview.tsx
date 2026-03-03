import { useEffect } from 'react'
import Prism from 'prismjs'

export function CookbookOverview() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Cookbook</h1>
                <p className="text-xl text-gray-500">
                    Concept-heavy recipes with explicit scaffold baseline and optional extension paths.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Legend</h2>
                <ul>
                    <li>
                        <strong>Scaffold Now (verified)</strong>: files that exist in generated projects from{' '}
                        <code>scaffold/template</code>.
                    </li>
                    <li>
                        <strong>Concept Extension (optional)</strong>: recommended structure for new
                        modules/features not created by scaffold default.
                    </li>
                </ul>

                <h2>Framework Single Sources of Truth</h2>
                <ul>
                    <li>
                        Schema SSOT: <code>app/schemas/*.toml</code>
                    </li>
                    <li>
                        Permission SSOT: <code>app/permissions.toml</code>
                    </li>
                    <li>
                        Contract SSOT: <code>app/src/contracts/api/v1/*</code> +{' '}
                        <code>app/src/contracts/datatable/*</code>
                    </li>
                    <li>
                        Datatable runtime SSOT catalog:{' '}
                        <code>app/src/internal/datatables/v1/admin/mod.rs</code>
                    </li>
                    <li>
                        Route composer: <code>app/src/internal/api/v1/mod.rs</code>
                    </li>
                    <li>
                        Jobs registry: <code>app/src/internal/jobs/mod.rs</code>
                    </li>
                    <li>
                        Console entrypoint: <code>app/src/bin/console.rs</code>
                    </li>
                </ul>

                <h2>Chapter Contract</h2>
                <ul>
                    <li>Each chapter includes Objective.</li>
                    <li>Each chapter starts with Scaffold Now (verified).</li>
                    <li>Concept Extension is clearly marked when used.</li>
                    <li>Each chapter ends with Verify / run commands.</li>
                </ul>

                <h2>Read Order</h2>
                <ol>
                    <li>Chapter 1, Chapter 2A, Chapter 2B (contracts + auth baseline).</li>
                    <li>Chapter 3, Chapter 7 (jobs + operational CLI baseline).</li>
                    <li>Chapter 5, Chapter 6 (realtime baseline then policy extension).</li>
                    <li>Chapter 8, Chapter 9 (system integration + hardening).</li>
                    <li>Chapter 10, Chapter 11, Chapter 12 (advanced recipes).</li>
                </ol>

                <h2>Chapter Links</h2>
                <div className="not-prose grid grid-cols-1 gap-3">
                    <a href="#/cookbook-chapter-1-crud-api-server" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 1: CRUD API Baseline</a>
                    <a href="#/cookbook-chapter-2-validation-dto" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 2A: DTO + Validation</a>
                    <a href="#/cookbook-chapter-2-admin-auth" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 2B: Admin Auth</a>
                    <a href="#/cookbook-chapter-3-jobs-usage" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 3: Jobs</a>
                    <a href="#/cookbook-chapter-4-notifications" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 4: Notifications</a>
                    <a href="#/cookbook-chapter-5-websocket-channel" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 5: WebSocket Channel</a>
                    <a href="#/cookbook-chapter-6-websocket-auth-middleware" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 6: WebSocket Auth Policy</a>
                    <a href="#/cookbook-chapter-7-cli-workflow" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 7: CLI Workflow</a>
                    <a href="#/cookbook-chapter-8-end-to-end-flow" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 8: End-to-End Flow</a>
                    <a href="#/cookbook-chapter-9-production-hardening" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 9: Production Hardening</a>
                    <a href="#/cookbook-chapter-10-caching" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 10: Caching</a>
                    <a href="#/cookbook-chapter-11-testing" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 11: Testing</a>
                    <a href="#/cookbook-chapter-12-event-fanout" className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800">Chapter 12: Event Fan-out</a>
                </div>
            </div>
        </div>
    )
}
