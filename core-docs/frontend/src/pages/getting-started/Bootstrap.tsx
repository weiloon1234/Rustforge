export function Bootstrap() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Bootstrap</h1>
                <p className="text-xl text-gray-500">Unified initialization for every runtime.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    <code>bootstrap</code> is the single startup path for API, websocket, worker,
                    and console. It centralizes env, logging, database, redis, and i18n setup.
                </p>

                <h3>Initialization Flow</h3>
                <ul className="list-decimal pl-5 space-y-2 text-gray-700">
                    <li>Load environment and base settings.</li>
                    <li>Initialize tracing/log output.</li>
                    <li>Connect Postgres and Redis.</li>
                    <li>Load i18n and runtime services.</li>
                </ul>

                <h3 className="mt-8">API Binary</h3>
                <p>Starter file: <code>app/src/bin/api-server.rs</code></p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::web::start_server(
        app::internal::api::build_router,
        |ctx| async move {
            bootstrap::jobs::start_with_context(
                ctx,
                app::internal::jobs::register_jobs,
                Some(app::internal::jobs::register_schedules),
            )
            .await
        },
    )
    .await
}`}</code>
                </pre>
            </div>
        </div>
    )
}
