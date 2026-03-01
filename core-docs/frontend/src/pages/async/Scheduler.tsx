export function Scheduler() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Scheduler</h1>
                <p className="text-xl text-gray-500">Cron-like scheduling on top of the job worker.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    App schedules are registered in <code>app/src/internal/jobs/mod.rs</code> via{' '}
                    <code>register_schedules</code>.
                </p>

                <h3>Scaffold baseline</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[allow(unused_variables)]
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {
    // add cron jobs here
}`}</code>
                </pre>

                <h3>Concept extension</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {
    scheduler
        .cron::<DailyCleanup>("0 0 0 * * *")
        .without_overlapping(300);
}`}</code>
                </pre>

                <p>
                    Scheduled items enqueue into the queue declared by each job&apos;s{' '}
                    <code>QUEUE</code> constant and are executed by worker processes.
                </p>
            </div>
        </div>
    )
}
