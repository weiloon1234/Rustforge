import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter8EndToEndFlow() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 8: End-to-End API + Jobs + Realtime Flow
                </h1>
                <p className="text-xl text-gray-500">
                    Compose a real baseline flow first, then layer fan-out patterns as extension.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Validate full request lifecycle from auth to response and optional async side effects.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ol>
                    <li>Admin login via <code>/api/v1/admin/auth/login</code>.</li>
                    <li>
                        Query datatable via <code>/api/v1/admin/datatable/admin/query</code>.
                    </li>
                    <li>
                        Query cross-page totals via <code>/api/v1/admin/datatable/admin/summary</code>.
                    </li>
                    <li>
                        Trigger export via <code>/api/v1/admin/datatable/admin/export/csv</code> or{' '}
                        <code>/export/email</code>.
                    </li>
                    <li>Track export status from datatable export status endpoint.</li>
                </ol>
                <ul>
                    <li>
                        API handlers: <code>app/src/internal/api/v1/admin/*.rs</code> +{' '}
                        <code>app/src/internal/api/datatable.rs</code>
                    </li>
                    <li>
                        Datatable contract: <code>app/src/contracts/datatable/admin/account.rs</code>
                    </li>
                    <li>
                        Datatable hooks: <code>app/src/internal/datatables/v1/admin/account.rs</code>
                    </li>
                    <li>
                        Datatable catalog (single register + mount source):{' '}
                        <code>app/src/internal/datatables/v1/admin/mod.rs</code>
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <p>
                    Add event fan-out when your domain needs background notifications or realtime broadcast.
                </p>
                <ul>
                    <li>
                        Add domain event jobs under <code>app/src/internal/jobs/</code> and register in{' '}
                        <code>app/src/internal/jobs/mod.rs</code>.
                    </li>
                    <li>
                        Publish realtime updates from job/workflow using <code>core_realtime::RealtimePublisher</code>.
                    </li>
                    <li>
                        Keep endpoint behavior deterministic even if fan-out channel is temporarily unavailable.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/api-server
./bin/worker
# 1) login and get token
# 2) call admin datatable query + summary
# 3) trigger export and poll status
# 4) optional: observe worker logs for async post-processing`}</code>
                </pre>
            </div>
        </div>
    )
}
