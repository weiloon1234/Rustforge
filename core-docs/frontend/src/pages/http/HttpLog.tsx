export function HttpLog() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">HTTP &amp; Webhook Logs</h1>
                <p className="text-xl text-gray-500">
                    Framework-owned logging for inbound webhooks and outbound HTTP client calls, with worker-driven cleanup.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where the SSOT lives</h2>
                <ul>
                    <li>
                        Runtime/env configuration in <code>.env</code> / typed settings
                    </li>
                    <li>
                        Framework implementation in <code>core-http-log</code>
                    </li>
                    <li>
                        App wiring via <code>WebhookLogLayer</code> and the framework <code>HttpClient</code> wrapper
                    </li>
                </ul>

                <h2>Retention</h2>
                <p>
                    Logs are cleaned up by the worker based on <code>HTTP_LOG_RETENTION_DAYS</code>. If the worker
                    is not running, old logs will accumulate.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-env">{`HTTP_LOG_RETENTION_DAYS=7
HTTP_LOG_WEBHOOK_ENABLED=false
HTTP_LOG_WEBHOOK_PATHS=/wh/,/webhook/
HTTP_LOG_CLIENT_ENABLED=false`}</code>
                </pre>

                <h2>Inbound webhook logging</h2>
                <p>
                    Use <code>WebhookLogLayer</code> for inbound third-party callbacks. Paths are matched by prefix,
                    not wildcard patterns.
                </p>
                <ul>
                    <li>
                        <code>/wh/</code> matches <code>/wh/stripe</code> and <code>/wh/slack</code>
                    </li>
                    <li>
                        <code>/wh/*</code> is treated literally and should not be used
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_http_log::WebhookLogLayer;

let app = Router::new()
    .route("/webhooks/stripe", post(handle_stripe))
    .layer(WebhookLogLayer::new(settings.http_log.clone()));`}</code>
                </pre>

                <h2>Outbound HTTP logging</h2>
                <p>
                    Use the framework <code>HttpClient</code> wrapper instead of raw <code>reqwest::Client</code> when
                    you want outbound request/response logging.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_http_log::HttpClient;

let client = HttpClient::new(&db, &settings.http_log);

let response = client
    .post("https://slack.com/api/chat.postMessage")
    .json(&payload)
    .send()
    .await?;`}</code>
                </pre>

                <h2>Use cases</h2>
                <ul>
                    <li>Audit external integration traffic</li>
                    <li>Debug webhook retries and bad payloads</li>
                    <li>Track outbound dependency behavior without custom logging wrappers</li>
                </ul>

                <h2>Operational rule</h2>
                <p>
                    Keep the logging mechanism framework-owned, but keep the decision about which paths or outbound
                    calls are log-worthy in app configuration and app wiring. Do not duplicate a second custom audit
                    log unless you actually need domain-specific semantics that the framework log does not cover.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/jobs">Jobs &amp; Queue</a> for worker/runtime ownership of cleanup.
                    </li>
                    <li>
                        <a href="#/cookbook/add-notifications">Add Notifications</a> or other integration recipes when outbound API calls are part of a workflow.
                    </li>
                </ul>
            </div>
        </div>
    )
}
