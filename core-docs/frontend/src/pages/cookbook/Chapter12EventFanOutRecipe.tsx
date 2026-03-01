import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter12EventFanOutRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 12: Event Fan-out Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Fan out one domain event into jobs, notifications, and realtime channels with controlled failure domains.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Use queue-backed orchestration for non-critical side effects while preserving API response predictability.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Worker registry and schedule registry are in <code>app/src/internal/jobs/mod.rs</code>.
                    </li>
                    <li>
                        Realtime runtime exists via websocket binary + internal realtime module.
                    </li>
                    <li>
                        Contracts/permissions remain isolated from fan-out mechanism.
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add event-specific job modules under <code>app/src/internal/jobs/</code> (for example{' '}
                        <code>order_placed_fanout.rs</code> or <code>article_published_fanout.rs</code>).
                    </li>
                    <li>
                        Dispatch from workflow after main transaction commit (or with outbox buffer if atomicity required).
                    </li>
                    <li>
                        Keep each channel adapter isolated: email/push/realtime in separate functions or jobs.
                    </li>
                    <li>
                        Add idempotency key checks for at-least-once delivery safety.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/api-server
./bin/worker
./bin/websocket-server
# trigger domain endpoint
# verify: API success, worker fan-out execution, optional realtime delivery`}</code>
                </pre>
            </div>
        </div>
    )
}
