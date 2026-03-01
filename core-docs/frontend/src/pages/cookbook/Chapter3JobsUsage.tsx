import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter3JobsUsage() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 3: Job Creation and Usage Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Register jobs once, dispatch from API/workflow, and process through worker with outbox safety.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Add background processing without breaking transaction integrity or route/contract separation.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Jobs registry file: <code>app/src/internal/jobs/mod.rs</code>
                    </li>
                    <li>
                        Worker binary: <code>app/src/bin/worker.rs</code>
                    </li>
                    <li>
                        API server optional embedded worker boot: <code>app/src/bin/api-server.rs</code>
                    </li>
                    <li>
                        Worker wiring uses <code>register_jobs</code> + <code>register_schedules</code>
                    </li>
                </ul>

                <h3>Registry baseline</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
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
                        Add concrete job modules under <code>app/src/internal/jobs/</code> (for example{' '}
                        <code>app/src/internal/jobs/rebuild_index.rs</code>).
                    </li>
                    <li>
                        Use <code>JobBuffer</code> when enqueue must commit atomically with DB writes.
                    </li>
                    <li>
                        Separate queue names by workload (<code>default</code>, <code>mail</code>,{' '}
                        <code>realtime</code>, <code>maintenance</code>).
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p app
RUN_WORKER=true ./bin/api-server
# or run dedicated worker:
./bin/worker`}</code>
                </pre>
            </div>
        </div>
    )
}
