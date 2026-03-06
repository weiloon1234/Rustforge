export function Jobs() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Job Queue</h1>
                <p className="text-xl text-gray-500">
                    Durable queue processing with explicit worker registration, outbox-backed dispatch, failed-job inspection, and optional in-process runtime fan-out.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What the jobs system is for</h2>
                <p>
                    Use queued jobs when work must survive process restarts and run outside the request lifecycle. Keep request handlers deterministic; hand slow or retryable work to the job system.
                </p>

                <h2>Main runtime pieces</h2>
                <ul>
                    <li><strong>Job type:</strong> implement <code>core_jobs::Job</code> for your payload and handler logic.</li>
                    <li><strong>Worker registry:</strong> register jobs in <code>app/src/internal/jobs/mod.rs</code>.</li>
                    <li><strong>Worker runtime:</strong> <code>app/src/bin/worker.rs</code> or embedded worker startup from the API server.</li>
                    <li><strong>Failed jobs:</strong> framework-owned failed job storage and retry/list APIs.</li>
                    <li><strong>Outbox:</strong> use durable buffering when enqueue must commit atomically with DB writes.</li>
                </ul>

                <h2>Scaffold registration shape</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub fn register_jobs(worker: &mut Worker) {
    // worker.register::<YourJob>();
}

#[allow(unused_variables)]
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {}`}</code>
                </pre>

                <h2>Outbox and failed-job behavior</h2>
                <ul>
                    <li>Use durable enqueue when the DB write and the future job must commit together.</li>
                    <li>Workers record terminal failures into <code>failed_jobs</code>.</li>
                    <li>The queue API can list failed jobs and retry them back into Redis.</li>
                </ul>

                <h2>When to use a queued job vs runtime fan-out</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Use queued job when</th>
                            <th>Use runtime async fan-out when</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>The work must survive process restarts.</td>
                            <td>The work is best-effort and local to the process.</td>
                        </tr>
                        <tr>
                            <td>You need retry/backoff and failed-job tracking.</td>
                            <td>You do not need queue durability or retry semantics.</td>
                        </tr>
                        <tr>
                            <td>The job should be processed by a separate worker fleet.</td>
                            <td>The work is cheap and tightly coupled to the current process runtime.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Practical rule</h2>
                <p>
                    Name the domain concept however you want, but the runtime primitive is still a job. Notifications, fan-out, maintenance, and rebuild work all eventually become queued jobs when they need durability.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/cookbook/add-jobs">Add Jobs</a> for the starter recipe.</li>
                    <li><a href="#/notifications">Notifications</a> for higher-level domain uses of the queue.</li>
                    <li><a href="#/scheduler">Scheduler &amp; Cron</a> for periodic dispatch patterns.</li>
                </ul>
            </div>
        </div>
    )
}
