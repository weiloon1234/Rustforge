export function RealtimeWebSocketFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Realtime / WebSocket</h1>
                <p className="text-xl text-gray-500">
                    Native WebSocket realtime module with guard reuse, Redis Pub/Sub fan-out, and
                    room presence.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What Framework Provides</h2>
                <ul>
                    <li>
                        Dedicated websocket binary in starter at
                        <code>app/src/bin/websocket-server.rs</code> (run via
                        <code>./bin/websocket-server</code>) using native WebSocket protocol.
                    </li>
                    <li>
                        Shared bootstrap and settings via <code>bootstrap::realtime::start_server</code>.
                    </li>
                    <li>
                        Guard reuse with PAT auth payload handshake through generated
                        <code>authenticate_any_guard</code>.
                    </li>
                    <li>
                        Global toggle plus per-channel enable/guard/presence policies.
                    </li>
                    <li>
                        Redis Pub/Sub fan-out from API/worker to websocket subscribers.
                    </li>
                    <li>
                        Presence TTL heartbeat per <code>channel + room + subject + connection</code>.
                    </li>
                    <li>
                        Optional durable replay mode via Redis Stream with per-event
                        <code>delivery_id</code> for client resume.
                    </li>
                </ul>

                <h2>Important Protocol Note</h2>
                <p>
                    This module uses <strong>native WebSocket</strong> only. Socket.IO clients are not
                    protocol-compatible unless you add a Socket.IO adapter.
                </p>
                <p>
                    For protocol state-machine and production runbook details, see
                    <a href="#/feature-realtime-protocol"> Realtime Protocol + Runbook</a>.
                </p>

                <h2>Config (Runtime .env)</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`REALTIME_ENABLED=true
REALTIME_HOST=0.0.0.0
REALTIME_PORT=3010
REALTIME_HEARTBEAT_SECS=20
REALTIME_PRESENCE_TTL_SECS=60
REALTIME_MAX_CONNECTIONS=10000
REALTIME_MAX_MESSAGE_BYTES=65536
REALTIME_MAX_FRAME_BYTES=65536
REALTIME_MAX_MESSAGES_PER_SEC=150
REALTIME_SEND_QUEUE_CAPACITY=1024
REALTIME_REQUIRE_AUTH=true
REALTIME_DELIVERY_MODE=at_most_once   # at_most_once | durable
REALTIME_STREAM_MAX_LEN=100000
REALTIME_STREAM_RETENTION_SECS=0
REALTIME_REPLAY_LIMIT_DEFAULT=200
REALTIME_REPLAY_LIMIT_MAX=1000
REALTIME_REPLAY_GAP_ALERT_THRESHOLD=100
REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS=60
REALTIME_CHECKPOINT_ENABLED=false
REALTIME_CHECKPOINT_TTL_SECS=2592000`}</code>
                </pre>
                <p>
                    In durable mode, publisher writes to Redis Stream and emits event
                    <code>delivery_id</code>. Client can pass <code>since_id</code> on subscribe to
                    replay missed events. If checkpoint is enabled, client can also ack processed
                    delivery IDs and reconnect without manually passing <code>since_id</code>.
                </p>

                <h2>Config (app/configs.toml)</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`# Optional channel registry
[realtime.channels.public_feed]
enabled = true
presence_enabled = false
max_messages_per_sec = 50

[realtime.channels.admin_notifications]
enabled = true
guard = "admin"   # example guard name only
presence_enabled = true
max_message_bytes = 32768
max_frame_bytes = 32768`}</code>
                </pre>
                <p>
                    Framework stays role-agnostic. Names like <code>admin</code> are examples only.
                </p>

                <h2>App Policy Hook (V3)</h2>
                <p>
                    Realtime policy checks are app-owned in
                    <code>app/src/internal/realtime_policy.rs</code>. Both websocket subscribe and
                    API publish pass a typed <code>PolicyContext</code> with identity, guard,
                    abilities, channel, room, source, and metadata.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_realtime::{PolicyContext, PolicyDecision, SubscribeAuthorizer};

impl SubscribeAuthorizer for AppRealtimePolicy {
  fn authorize_subscribe(&self, ctx: &PolicyContext) -> PolicyDecision {
    // Example: ability-scoped channel access
    if let Some(identity) = ctx.identity.as_ref() {
      let ability = format!("realtime:subscribe:{}", ctx.channel());
      if identity.can(&ability) || identity.can("realtime:subscribe:*") {
        return PolicyDecision::allow();
      }
    }
    PolicyDecision::allow()
  }
}`}</code>
                </pre>
                <p>
                    HTTP publish adapter fills policy metadata from headers
                    <code>x-request-id</code>, <code>x-forwarded-for</code>,
                    <code>user-agent</code>, and <code>x-tenant-id</code>.
                </p>
                <p>
                    WebSocket subscribe authorization now receives the same metadata extracted from
                    upgrade request headers, so policy decisions stay consistent between HTTP publish
                    and WS subscribe paths.
                </p>

                <h2>Client Protocol</h2>
                <p>Client to server ops:</p>
                <ul>
                    <li>
                        <code>auth</code>
                    </li>
                    <li>
                        <code>subscribe</code>
                    </li>
                    <li>
                        <code>unsubscribe</code>
                    </li>
                    <li>
                        <code>ping</code>
                    </li>
                    <li>
                        <code>ack</code>
                    </li>
                </ul>
                <p>Server to client ops:</p>
                <ul>
                    <li>
                        <code>auth_ok</code>
                    </li>
                    <li>
                        <code>event</code>
                    </li>
                    <li>
                        <code>presence</code>
                    </li>
                    <li>
                        <code>error</code>
                    </li>
                    <li>
                        <code>pong</code>
                    </li>
                    <li>
                        <code>ack_ok</code>
                    </li>
                    <li>
                        <code>replay_gap</code>
                    </li>
                </ul>

                <h2>Browser Example (Native WebSocket)</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

let lastDeliveryId = localStorage.getItem('rt:last_delivery_id');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({
    op: 'subscribe',
    channel: 'admin_notifications',
    room: 'agency:1',
    since_id: lastDeliveryId,
    replay_limit: 200
  }));
};

ws.onmessage = (evt) => {
  const msg = JSON.parse(evt.data);
  if (msg.op === 'event') {
    if (msg.delivery_id) {
      lastDeliveryId = msg.delivery_id;
      localStorage.setItem('rt:last_delivery_id', msg.delivery_id);
      ws.send(JSON.stringify({
        op: 'ack',
        channel: msg.channel,
        room: msg.room ?? null,
        delivery_id: msg.delivery_id
      }));
    }
    console.log(msg.channel, msg.event, msg.payload);
  } else if (msg.op === 'replay_gap') {
    console.warn('Replay gap detected', msg);
    // Client should refresh critical state or request a full sync.
  }
}; 

setInterval(() => {
  ws.send(JSON.stringify({ op: 'ping' }));
}, 20000);`}</code>
                </pre>

                <h2>Typed Publisher Usage</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};
use serde::Serialize;

#[derive(Serialize)]
struct AppointmentCreated {
    id: i64,
    title: String,
}

impl RealtimeEvent for AppointmentCreated {
    const CHANNEL: &'static str = "appointments";
    const EVENT: &'static str = "created";
}

let publisher = RealtimePublisher::from_realtime_settings(
    &settings.redis.url,
    &settings.realtime,
)?;
publisher
    .publish(
        RealtimeTarget { room: Some("agency:1".to_string()) },
        &AppointmentCreated { id: 42, title: "Demo".to_string() },
    )
    .await?;`}</code>
                </pre>

                <h2>Idempotency Helper (Handler-side)</h2>
                <p>
                    For durable replay/resume, use framework idempotency helper to skip duplicate
                    event handling by <code>delivery_id</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_realtime::RealtimeIdempotency;

let idem = RealtimeIdempotency::new(&settings.redis.url)?;

// scope can be your bounded handler domain
if idem.claim_once("notify:appointment-created", &delivery_id).await? {
    // first time -> process side effects
} else {
    // duplicate replay -> skip safely
}`}</code>
                </pre>

                <h2>Load Test Command (CLI)</h2>
                <p>
                    Use the built-in realtime benchmark command for fan-out and replay/ack stress
                    tests.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`./console realtime bench \\
  --ws-url ws://127.0.0.1:3010/ws \\
  --token <PAT_TOKEN> \\
  --channel public_feed \\
  --clients 100 \\
  --messages 1000 \\
  --publish-rate 500 \\
  --ack`}</code>
                </pre>
                <p>
                    Add <code>--json</code> for CI regression checks with machine-readable output.
                </p>
                <p>
                    Output includes connected/authenticated clients, published vs received counts,
                    delivery loss, replay-gap count, effective receive throughput, and latency
                    percentiles (p50/p95/p99).
                </p>

                <h2>Optional API Smoke Publish Endpoint</h2>
                <p>
                    App wiring includes <code>POST /admin/realtime/publish</code> for local/dev
                    smoke tests. It is blocked in production and requires
                    <code>x-realtime-key: APP_KEY</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`curl -X POST http://127.0.0.1:3000/admin/realtime/publish \\
  -H 'Content-Type: application/json' \\
  -H 'x-realtime-key: <APP_KEY>' \\
  -d '{"channel":"appointments","event":"created","room":"agency:1","payload":{"id":42}}'`}</code>
                </pre>

                <h2>Realtime Ops Endpoints (websocket-server)</h2>
                <ul>
                    <li>
                        <code>GET /health</code>
                    </li>
                    <li>
                        <code>GET /realtime/describe</code> (effective runtime/channel config)
                    </li>
                    <li>
                        <code>GET /realtime/metrics</code> (connections/auth/subscription/replay counters)
                    </li>
                    <li>
                        <code>GET /realtime/metrics.prom</code> (Prometheus exposition format
                        with channel/room labels, replay-gap counters, op/error labels, and latency
                        histograms)
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`curl http://127.0.0.1:3010/realtime/describe
curl http://127.0.0.1:3010/realtime/metrics
curl http://127.0.0.1:3010/realtime/metrics.prom`}</code>
                </pre>

                <h2>Delivery Semantics</h2>
                <p>
                    <code>REALTIME_DELIVERY_MODE=at_most_once</code> keeps fast
                    ephemeral fan-out.
                </p>
                <p>
                    <code>REALTIME_DELIVERY_MODE=durable</code> adds stream-backed replay by
                    <code>since_id</code>. This is still realtime-oriented and not a full message
                    queue replacement.
                </p>
                <p>
                    Retention controls: <code>REALTIME_STREAM_MAX_LEN</code> (count window) and
                    <code>REALTIME_STREAM_RETENTION_SECS</code> (time window). Replay-gap SLO
                    alerting uses <code>REALTIME_REPLAY_GAP_ALERT_THRESHOLD</code> and
                    <code>REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS</code>.
                </p>
                <p>
                    Outbound writes use bounded per-connection queue capacity
                    (<code>REALTIME_SEND_QUEUE_CAPACITY</code>). Slow consumers are disconnected
                    to prevent unbounded memory growth.
                </p>
            </div>
        </div>
    )
}
