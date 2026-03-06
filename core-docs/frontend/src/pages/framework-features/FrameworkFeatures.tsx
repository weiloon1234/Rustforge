import { useEffect } from 'react'
import Prism from 'prismjs'

const featureCards = [
    {
        href: '#/feature-autodatatable',
        title: 'AutoDataTable',
        description:
            'Typed datatable contracts, runtime hooks, and export/query parity through one pipeline.',
    },
    {
        href: '#/feature-meta',
        title: 'Meta',
        description:
            'JSONB-backed per-model key/value fields with generated typed readers and writers.',
    },
    {
        href: '#/feature-attachments',
        title: 'Attachments',
        description:
            'Attachment uploads, typed attachment DTOs, and hydrated URLs on model views.',
    },
    {
        href: '#/feature-localized-relations',
        title: 'Localized & Relationships',
        description:
            'Localized field storage, relation helpers, and relation-aware query loaders.',
    },
    {
        href: '#/feature-realtime',
        title: 'Realtime / WebSocket',
        description:
            'Guard-reused websocket auth, channel policies, presence, and Redis-backed fan-out.',
    },
    {
        href: '#/feature-realtime-protocol',
        title: 'Realtime Protocol & Runbook',
        description:
            'Wire protocol, replay behavior, operational failure modes, and room lifecycle rules.',
    },
]

export function FrameworkFeatures() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Framework Features</h1>
                <p className="text-xl text-gray-500">
                    Canonical documentation for shipped framework capabilities and their extension points.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <div className="rounded-xl border border-amber-200 bg-amber-50 px-5 py-4">
                    <p className="m-0 text-sm text-amber-900">
                        <strong>Documentation ownership:</strong> this docs app is the framework SSOT.
                        Root README stays operational. Starter-local guides stay in
                        <code> scaffold/template/docs/</code>.
                    </p>
                </div>

                <h2>What belongs here</h2>
                <p>
                    A framework feature page should document the capability itself: where the source
                    of truth lives, what runtime/API surface it exposes, what extension hooks exist,
                    and the minimal path to use it safely. Starter-only setup details should stay in
                    starter docs and be referenced by path when needed.
                </p>

                <h2>Feature anatomy</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Question</th>
                            <th>Expected answer on each feature page</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>What is the capability for?</td>
                            <td>Problem/intent and where it fits in the framework.</td>
                        </tr>
                        <tr>
                            <td>Where is the SSOT?</td>
                            <td>Schema/config/contract/runtime module that actually owns the behavior.</td>
                        </tr>
                        <tr>
                            <td>What does the runtime surface look like?</td>
                            <td>Generated model API, workflow call, router helper, or config contract.</td>
                        </tr>
                        <tr>
                            <td>Where do apps extend it?</td>
                            <td>Extension traits, app hooks, workflows, or starter-local docs.</td>
                        </tr>
                        <tr>
                            <td>How do I verify it?</td>
                            <td>Minimal example plus check/build or request flow.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Feature index</h2>
                <div className="not-prose grid grid-cols-1 gap-4 md:grid-cols-2">
                    {featureCards.map((card) => (
                        <a
                            key={card.href}
                            href={card.href}
                            className="rounded-xl border border-gray-200 bg-white p-5 transition-colors hover:bg-gray-50"
                        >
                            <h3 className="m-0 text-base font-semibold text-gray-900">{card.title}</h3>
                            <p className="mt-2 text-sm text-gray-600">{card.description}</p>
                        </a>
                    ))}
                </div>

                <h2>Shipped capability map</h2>
                <ul>
                    <li>
                        <strong>Model features:</strong> meta, attachments, localized fields, relations,
                        computed/extension traits, enum explained fields.
                    </li>
                    <li>
                        <strong>Transport features:</strong> typed contracts, guard/authz helpers,
                        OpenAPI integration, bootstrap runtime injection.
                    </li>
                    <li>
                        <strong>Operational features:</strong> jobs, notifications, caching, realtime,
                        HTTP/webhook logging, datatable export.
                    </li>
                </ul>

                <h2>Starter handoff points</h2>
                <p>
                    When a workflow becomes starter-specific, hand off to starter docs instead of
                    duplicating it here. Current starter-local examples include:
                </p>
                <ul>
                    <li>
                        <code>scaffold/template/docs/computed-model-values.md</code> for app-facing
                        model/view extension patterns.
                    </li>
                    <li>
                        <code>scaffold/template/docs/country-iso2-linkage.md</code> for country
                        linkage policy and migration playbook.
                    </li>
                    <li>
                        <code>scaffold/template/app/AGENTS.md</code> for starter-specific extension
                        recipes and file layout conventions.
                    </li>
                </ul>

                <h2>Where to go next</h2>
                <ul>
                    <li>
                        Read <a href="#/requests">Requests &amp; Validation</a> for contract
                        boundary semantics such as <code>Option&lt;T&gt;</code> and{' '}
                        <code>Patch&lt;T&gt;</code>.
                    </li>
                    <li>
                        Read <a href="#/model-api-view">`XxxView` &amp; Extensions</a> for computed
                        values and typed app-facing model helpers.
                    </li>
                    <li>
                        Read <a href="#/cookbook">Cookbook</a> for task-oriented implementation recipes.
                    </li>
                </ul>
            </div>
        </div>
    )
}
