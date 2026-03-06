export function RealtimeWebSocketFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Realtime / WebSocket</h1>
                <p className="text-xl text-gray-500">
                    Native websocket runtime with framework-owned protocol/publisher primitives and
                    app-owned channel policy.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Responsibility split</h2>
                <ul>
                    <li>
                        <strong>Framework-owned:</strong> websocket protocol, publisher, replay,
                        presence, metrics, and <code>WsServerState</code>.
                    </li>
                    <li>
                        <strong>App-owned:</strong> channel names, subscribe/publish authorization
                        rules beyond the framework baseline, and which workflows/jobs emit events.
                    </li>
                </ul>

                <h2>Starter baseline</h2>
                <ul>
                    <li>websocket server binary: <code>app/src/bin/websocket-server.rs</code></li>
                    <li>app realtime module: <code>app/src/internal/realtime/</code></li>
                    <li>runtime env: <code>REALTIME_*</code></li>
                    <li>channel config: <code>app/configs.toml</code></li>
                    <li>generated any-guard token resolution for websocket auth</li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`REALTIME_ENABLED=true
REALTIME_HOST=0.0.0.0
REALTIME_PORT=3010
REALTIME_REQUIRE_AUTH=true
REALTIME_DELIVERY_MODE=at_most_once`}</code>
                </pre>

                <h2>Framework-enforced baseline</h2>
                <p>
                    The realtime server already enforces global enabled state, per-channel enabled
                    state, auth-required behavior, and configured guard matching before app-specific
                    subscribe policy runs.
                </p>

                <h2>Client baseline</h2>
                <p>
                    The framework uses a native websocket protocol with ops such as <code>auth</code>,{' '}
                    <code>subscribe</code>, <code>unsubscribe</code>, <code>ping</code>, and{' '}
                    <code>ack</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({
    op: 'subscribe',
    channel: 'admin_notifications',
    room: 'country:admin',
  }));
};`}</code>
                </pre>

                <h2>Publisher pattern</h2>
                <p>
                    Publish from workflows or jobs through the framework publisher. The event type
                    and target room remain app-defined.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};

#[derive(serde::Serialize)]
struct AdminChanged {
    id: i64,
}

impl RealtimeEvent for AdminChanged {
    const CHANNEL: &'static str = "admin_notifications";
    const EVENT: &'static str = "updated";
}

let publisher = RealtimePublisher::from_realtime_settings(&settings.redis.url, &settings.realtime)?;
publisher
    .publish(
        RealtimeTarget::room("tenant:1"),
        &AdminChanged { id: 1 },
    )
    .await?;`}</code>
                </pre>

                <h2>Policy extension point</h2>
                <p>
                    The starter now ships a real websocket router that builds <code>WsServerState</code>
                    from configured channels, generated any-guard token resolution, and default
                    allow-all app authorizers. Keep custom subscribe/publish authorization checks
                    under <code>app/src/internal/realtime/</code> rather than hardcoding them in
                    transport handlers.
                </p>

                <h2>Operational endpoints in the starter websocket server</h2>
                <ul>
                    <li><code>GET /health</code></li>
                    <li><code>GET /realtime/metrics</code></li>
                    <li><code>GET /ws</code> websocket upgrade route</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/feature-realtime-protocol">Realtime Protocol &amp; Runbook</a>{' '}
                        for the wire contract and incident guidance.
                    </li>
                    <li>
                        <a href="#/cookbook/add-realtime-channel">Add a Realtime Channel</a> for
                        the starter recipe.
                    </li>
                    <li>
                        <a href="#/cookbook/add-websocket-auth">Add WebSocket Auth</a> for
                        auth-specific recipe steps.
                    </li>
                </ul>
            </div>
        </div>
    )
}
