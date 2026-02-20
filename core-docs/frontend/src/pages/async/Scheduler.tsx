export function Scheduler() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Scheduler</h1>
                <p className="text-xl text-gray-500">Cron-like task scheduling.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Cron schedules are registered in <code>app/src/internal/jobs/lib.rs</code> via
                    <code>register_schedules</code>. This is similar to Laravel&apos;s scheduler,
                    but due tasks are enqueued into Redis and executed by workers.
                </p>

                <h3>Register schedules</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_jobs::cron::Scheduler;

pub fn register_schedules(scheduler: &mut Scheduler) {
    scheduler
        .cron::<DailyCleanup>("0 0 0 * * *")
        .without_overlapping(300);

    scheduler
        .cron::<HourlySync>("0 0 * * * *")
        .when(|| !cfg!(debug_assertions));
}`}</code>
                </pre>

                <h3>Queue routing</h3>
                <p>
                    Scheduled jobs are dispatched to the queue declared by each job&apos;s
                    <code>QUEUE</code> constant.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`impl Job for DailyCleanup {
    const NAME: &'static str = "DailyCleanup";
    const QUEUE: &'static str = "maintenance";
    // ...
}`}</code>
                </pre>

                <h3>Bootstrap integration</h3>
                <p>
                    Framework schedules (for example HTTP log cleanup) are auto-registered in
                    <code>bootstrap::jobs</code>; your app schedules are appended on top.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`bs::jobs::start_with_context(
    ctx,
    register_jobs,
    Some(register_schedules),
).await?;`}</code>
                </pre>
            </div>
        </div>
    )
}
