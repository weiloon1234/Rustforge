import { useEffect } from 'react'
import Prism from 'prismjs'

const recipeLinks = [
    ['#/cookbook/build-crud-admin-resource', 'Build a CRUD Admin Resource'],
    ['#/cookbook/add-admin-datatable', 'Add an Admin DataTable'],
    ['#/cookbook/add-validation-contracts', 'Add Validation Contracts'],
    ['#/cookbook/add-admin-auth-permission-gates', 'Add Admin Auth & Permission Gates'],
    ['#/cookbook/add-jobs', 'Add Jobs'],
    ['#/cookbook/add-notifications', 'Add Notifications'],
    ['#/cookbook/add-realtime-channel', 'Add a Realtime Channel'],
    ['#/cookbook/add-websocket-auth', 'Add WebSocket Auth'],
    ['#/cookbook/add-console-workflow', 'Add a Console Workflow'],
    ['#/cookbook/build-end-to-end-flow', 'Build an End-to-End Flow'],
    ['#/cookbook/production-hardening', 'Production Hardening'],
    ['#/cookbook/add-caching', 'Add Caching'],
    ['#/cookbook/test-the-flow', 'Test the Flow'],
    ['#/cookbook/fan-out-events', 'Fan-out Events'],
]

export function CookbookOverview() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Cookbook</h1>
                <p className="text-xl text-gray-500">
                    Task-oriented implementation recipes that start from real scaffold conventions and
                    hand off to starter-specific guides only when necessary.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>How to use the cookbook</h2>
                <p>
                    The cookbook is no longer a chapter-first tutorial. Each page is a concrete job. Start with
                    the recipe you are actually trying to implement, then follow the cross-links into Feature,
                    HTTP/API, or Database docs when you need the lower-level framework contract.
                </p>

                <h2>Framework and starter boundaries</h2>
                <ul>
                    <li>
                        <strong>Framework docs:</strong> describe shipped capabilities, typed APIs, and shared
                        runtime behavior.
                    </li>
                    <li>
                        <strong>Starter docs:</strong> explain project-local setup and migration playbooks under
                        <code> scaffold/template/docs/</code>.
                    </li>
                    <li>
                        <strong>AGENTS:</strong> describe contributor extension recipes for the generated starter.
                    </li>
                </ul>

                <h2>Single sources of truth</h2>
                <ul>
                    <li>
                        Model SSOT: <code>app/models/*.rs</code>
                    </li>
                    <li>
                        Permission SSOT: <code>app/permissions.toml</code>
                    </li>
                    <li>
                        Contract SSOT: <code>app/src/contracts/api/v1/*</code> and{' '}
                        <code>app/src/contracts/datatable/*</code>
                    </li>
                    <li>
                        Generated view method surface: <code>app/models/*.rs</code> via <code>#[rf_view_impl]</code>
                    </li>
                    <li>
                        Starter-only operational guides: <code>scaffold/template/docs/*</code>
                    </li>
                </ul>

                <h2>Recommended reading order</h2>
                <ol>
                    <li>
                        <a href="#/cookbook/build-crud-admin-resource">Build a CRUD Admin Resource</a>
                    </li>
                    <li>
                        <a href="#/cookbook/add-admin-datatable">Add an Admin DataTable</a>
                    </li>
                    <li>
                        <a href="#/cookbook/add-validation-contracts">Add Validation Contracts</a>
                    </li>
                    <li>
                        <a href="#/cookbook/add-admin-auth-permission-gates">Add Admin Auth &amp; Permission Gates</a>
                    </li>
                    <li>Then jump directly to end-to-end flow, jobs, realtime, caching, or testing based on the job at hand.</li>
                </ol>

                <h2>Recipe index</h2>
                <div className="not-prose grid grid-cols-1 gap-3">
                    {recipeLinks.map(([href, label]) => (
                        <a
                            key={href}
                            href={href}
                            className="rounded-md border border-gray-200 bg-white px-4 py-3 text-sm font-medium text-gray-800 hover:bg-gray-50"
                        >
                            {label}
                        </a>
                    ))}
                </div>

                <h2>Starter handoff guides</h2>
                <ul>
                    <li>
                        <code>scaffold/template/docs/computed-model-values.md</code> for view/model extension examples.
                    </li>
                    <li>
                        <code>scaffold/template/docs/country-iso2-linkage.md</code> for country linkage and migration rules.
                    </li>
                    <li>
                        <code>scaffold/template/docs/custom-project-commands.md</code> for project-specific console commands.
                    </li>
                </ul>
            </div>
        </div>
    )
}
