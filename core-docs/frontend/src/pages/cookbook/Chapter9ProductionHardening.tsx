import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter9ProductionHardening() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 9: Production Hardening Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Harden auth, datatable export, and async delivery paths for predictable behavior in production.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Reduce operational risk by making retries, authorization boundaries, and observability explicit.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>Auth guard + permissions catalog are already wired.</li>
                    <li>
                        Datatable export endpoints and status checks are available in scaffold admin portal.
                    </li>
                    <li>
                        Job infrastructure exists through <code>app/src/internal/jobs/mod.rs</code> and worker binary.
                    </li>
                    <li>
                        Realtime runtime can be enabled/disabled by <code>REALTIME_ENABLED</code>.
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>Add idempotency keys for handlers and event consumers.</li>
                    <li>Define explicit retry caps and dead-letter policy for each job class.</li>
                    <li>
                        Add replay-gap handling and durable mode tuning for realtime if strict delivery is required.
                    </li>
                    <li>Add health/metrics checks in deployment probes and alerts.</li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./console route list --json
curl -sS http://127.0.0.1:3010/health
curl -sS http://127.0.0.1:3010/realtime/metrics
# run a datatable export and verify status lifecycle`}</code>
                </pre>
            </div>
        </div>
    )
}
