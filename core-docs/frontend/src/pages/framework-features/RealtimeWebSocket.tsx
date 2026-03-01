export function RealtimeWebSocketFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Realtime / WebSocket</h1>
                <p className="text-xl text-gray-500">
                    Native WebSocket runtime with shared bootstrap settings and optional app policy extensions.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Realtime server binary exists at <code>app/src/bin/websocket-server.rs</code>
                    </li>
                    <li>
                        Internal module placeholder exists at <code>app/src/internal/realtime/mod.rs</code>
                    </li>
                    <li>
                        Runtime options are controlled by <code>REALTIME_*</code> env vars
                    </li>
                    <li>
                        Channel config is app-owned in <code>app/configs.toml</code>
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`REALTIME_ENABLED=true
REALTIME_HOST=0.0.0.0
REALTIME_PORT=3010
REALTIME_REQUIRE_AUTH=true
REALTIME_DELIVERY_MODE=at_most_once`}</code>
                </pre>

                <h2>Protocol Baseline</h2>
                <p>
                    Use native WebSocket protocol with these ops: <code>auth</code>, <code>subscribe</code>,{' '}
                    <code>unsubscribe</code>, <code>ping</code>, and <code>ack</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({ op: 'subscribe', channel: 'admin_notifications' }));
};`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add explicit policy modules under <code>app/src/internal/realtime/</code> and plug into
                        subscribe/publish authorizers.
                    </li>
                    <li>
                        Add HTTP publish endpoint only for internal tooling or smoke test; it is not scaffold default.
                    </li>
                    <li>
                        Use durable mode + replay checkpoint when client resume guarantees are needed.
                    </li>
                </ul>

                <h2>Publisher Pattern</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};

#[derive(serde::Serialize)]
struct AdminChanged { id: i64 }

impl RealtimeEvent for AdminChanged {
    const CHANNEL: &'static str = "admin_notifications";
    const EVENT: &'static str = "updated";
}

let publisher = RealtimePublisher::from_realtime_settings(&settings.redis.url, &settings.realtime)?;
publisher
    .publish(RealtimeTarget { room: Some("tenant:1".to_string()) }, &AdminChanged { id: 1 })
    .await?;`}</code>
                </pre>

                <h2>Ops Endpoints</h2>
                <ul>
                    <li><code>GET /health</code></li>
                    <li><code>GET /realtime/describe</code></li>
                    <li><code>GET /realtime/metrics</code></li>
                    <li><code>GET /realtime/metrics.prom</code></li>
                </ul>

                <h2>Run</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`./bin/websocket-server
curl -sS http://127.0.0.1:3010/health
curl -sS http://127.0.0.1:3010/realtime/metrics`}</code>
                </pre>
            </div>
        </div>
    )
}
