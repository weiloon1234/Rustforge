export function HttpLog() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">HTTP Logging</h1>
                <p className="text-xl text-gray-500">
                    Audit log for inbound webhooks and outbound HTTP client requests.
                </p>

                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mt-2">
                    <p className="text-sm text-blue-700">
                        <strong>Retention Policy:</strong> Logs are automatically cleaned up by the background worker.
                        Default retention is <strong>7 days</strong>.
                        <br/>
                        Configure this via <code>HTTP_LOG_RETENTION_DAYS</code> in your <code>.env</code> file.
                    </p>
                </div>
            </div>
            <div className="prose prose-orange max-w-none">
                <p>
                    The framework provides a unified system for logging HTTP traffic, useful for auditing external integrations.
                </p>

                {/* Global Configuration & Retention */}
                <h2 className="mt-8">Global Configuration</h2>
                <div className="bg-gray-50 border rounded-lg p-6 my-4">
                    <h4 className="text-gray-900 font-bold m-0 mb-2">Retention Policy</h4>
                    <p className="text-sm text-gray-600 mb-2">
                        Logs are <strong>automatically cleaned up</strong> by the background worker.
                    </p>
                    <ul className="list-disc pl-5 text-sm text-gray-600 space-y-1 mb-4">
                        <li>Default retention: <strong>7 days</strong></li>
                        <li>Worker process must be running (<code>RUN_WORKER=true</code>)</li>
                    </ul>
                    <div className="bg-gray-900 text-gray-100 p-3 rounded text-sm font-mono overflow-x-auto">
                        HTTP_LOG_RETENTION_DAYS=7
                    </div>
                </div>

                <hr className="my-8 border-gray-200" />

                {/* Inbound Webhooks */}
                <h2>Inbound Webhooks</h2>
                <p>
                    Log incoming requests from third-party services (Stripe, Slack, etc).
                </p>

                <h3 className="text-lg font-bold text-gray-900 mt-4">1. Enable Logging</h3>
                <pre className="bg-gray-900 text-gray-100 p-3 rounded text-sm font-mono overflow-x-auto block mt-2">
                    HTTP_LOG_WEBHOOK_ENABLED=true
                </pre>

                <h3 className="text-lg font-bold text-gray-900 mt-6">2. Configure Paths</h3>
                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mt-2">
                    <p className="text-sm text-blue-700">
                        <strong>Matching Rule:</strong> Paths are matched by <strong>Prefix</strong>.
                    </p>
                    <p className="text-sm text-blue-700 mt-1">
                        <span className="font-bold text-red-600">⚠ No Wildcards:</span> Do not use <code>*</code>. To match <code>/webhooks/stripe</code> and <code>/webhooks/slack</code>, just use <code>/webhooks/</code>.
                    </p>
                </div>
                
                <table className="min-w-full text-sm border-collapse border border-gray-200 mt-4">
                    <thead className="bg-gray-100">
                        <tr>
                            <th className="border p-2 text-left">Config Value</th>
                            <th className="border p-2 text-left">Matches</th>
                            <th className="border p-2 text-left">Does NOT Match</th>
                        </tr>
                    </thead>
                    <tbody>
                         <tr>
                            <td className="border p-2 font-mono text-blue-600">/wh/</td>
                            <td className="border p-2">/wh/stripe, /wh/slack</td>
                            <td className="border p-2">/api/wh</td>
                        </tr>
                        <tr>
                            <td className="border p-2 font-mono text-red-600">/wh/*</td>
                            <td className="border p-2">/wh/* (Literal only)</td>
                            <td className="border p-2">/wh/stripe</td>
                        </tr>
                    </tbody>
                </table>

                <pre className="bg-gray-900 text-gray-100 p-3 rounded text-sm font-mono overflow-x-auto block mt-4">
                    HTTP_LOG_WEBHOOK_PATHS="/webhooks/,/callbacks/"
                </pre>

                <hr className="my-8 border-gray-200" />

                {/* Outbound Requests */}
                <h2>Outbound Log (HttpClient)</h2>
                <p>
                    Log outgoing API requests made by your application.
                </p>

                 <h3 className="text-lg font-bold text-gray-900 mt-4">1. Enable Logging</h3>
                <pre className="bg-gray-900 text-gray-100 p-3 rounded text-sm font-mono overflow-x-auto block mt-2">
                    HTTP_LOG_CLIENT_ENABLED=true
                </pre>

                <h3 className="text-lg font-bold text-gray-900 mt-6">2. Use HttpClient</h3>
                <p className="text-sm text-gray-600 mb-2">
                    You must use the framework's <code>HttpClient</code> wrapper instead of raw <code>reqwest</code>.
                </p>
                 <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_http_log::HttpClient;

pub async fn call_slack(db: &DbConn, settings: &Settings) -> Result<()> {
    // Client automatically mimics reqwest::Client but handles logging
    let client = HttpClient::new(db, &settings.http_log);

    let res = client.post("https://slack.com/api/chat.postMessage")
        .json(&payload)
        .send()
        .await?;
        
    Ok(())
}`}</code>
                </pre>
            </div>
        </div>
    )
}
