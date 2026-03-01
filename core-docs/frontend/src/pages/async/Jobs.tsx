export function Jobs() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Job Queue</h1>
                <p className="text-xl text-gray-500">
                    Durable Redis-backed jobs plus lightweight in-process runtime tasks.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        App job registration entrypoint is <code>app/src/internal/jobs/mod.rs</code>
                    </li>
                    <li>
                        Worker process entrypoint is <code>app/src/bin/worker.rs</code>
                    </li>
                    <li>
                        API binary can run embedded worker depending on runtime config
                    </li>
                </ul>

                <h3>Registry baseline</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_jobs::worker::Worker;

pub fn register_jobs(worker: &mut Worker) {
    // worker.register::<YourJob>();
}

#[allow(unused_variables)]
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {}`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Implement <code>core_jobs::Job</code> for domain jobs under{' '}
                        <code>app/src/internal/jobs/</code>.
                    </li>
                    <li>
                        Use <code>core_jobs::buffer::JobBuffer</code> for transactional enqueue with outbox safety.
                    </li>
                    <li>
                        Use <code>core_jobs::runtime</code> helpers for in-process async fan-out that does not need
                        Redis durability.
                    </li>
                </ul>
            </div>
        </div>
    )
}
