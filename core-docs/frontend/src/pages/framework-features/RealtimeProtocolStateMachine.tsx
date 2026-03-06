export function RealtimeProtocolStateMachineFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Realtime Protocol &amp; Runbook</h1>
                <p className="text-xl text-gray-500">
                    Wire-level websocket contract, delivery semantics, and operational failure handling.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Protocol contract</h2>
                <p>Client-to-server operations:</p>
                <ul>
                    <li><code>auth</code></li>
                    <li><code>subscribe</code></li>
                    <li><code>unsubscribe</code></li>
                    <li><code>ack</code></li>
                    <li><code>ping</code></li>
                </ul>
                <p>Server-to-client operations:</p>
                <ul>
                    <li><code>auth_ok</code></li>
                    <li><code>event</code></li>
                    <li><code>replay_gap</code></li>
                    <li><code>presence</code></li>
                    <li><code>ack_ok</code></li>
                    <li><code>error</code></li>
                    <li><code>pong</code></li>
                </ul>

                <h2>Event envelope</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-json">{`{
  "op": "event",
  "channel": "public_feed",
  "event": "updated",
  "room": "tenant:42",
  "payload": { "...": "..." },
  "sent_at_unix_ms": 1739952000123,
  "delivery_id": "1739952000123-0"
}`}</code>
                </pre>
                <p>
                    The timestamp contract is <code>sent_at_unix_ms</code>. The protocol should document only the
                    current envelope, not older timestamp layouts.
                </p>

                <h2>Connection state machine</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-mermaid">{`stateDiagram-v2
    [*] --> CONNECTED
    CONNECTED --> AUTHENTICATED: auth (valid token)
    CONNECTED --> CLOSED: auth timeout / invalid auth
    AUTHENTICATED --> SUBSCRIBED: subscribe (allowed)
    AUTHENTICATED --> AUTHENTICATED: subscribe denied (error)
    SUBSCRIBED --> SUBSCRIBED: event / presence / ping-pong
    SUBSCRIBED --> SUBSCRIBED: ack -> checkpoint update
    SUBSCRIBED --> AUTHENTICATED: unsubscribe
    SUBSCRIBED --> CLOSED: slow consumer disconnect / network close
    AUTHENTICATED --> CLOSED: network close
    CLOSED --> [*]`}</code>
                </pre>

                <h2>Delivery modes and replay</h2>
                <ul>
                    <li>
                        <strong>at_most_once:</strong> fire-and-forget websocket delivery without replay.
                    </li>
                    <li>
                        <strong>durable:</strong> Redis Stream backed delivery with <code>delivery_id</code> and resume support.
                    </li>
                    <li>
                        Client resume can use <code>since_id</code> or prior ack checkpoint.
                    </li>
                    <li>
                        If the requested cursor is outside retention, the server emits <code>replay_gap</code>.
                    </li>
                </ul>

                <h2>Production runbook</h2>
                <h3>Security</h3>
                <ul>
                    <li>Keep <code>REALTIME_REQUIRE_AUTH=true</code> in production.</li>
                    <li>Do not place access tokens in query strings.</li>
                    <li>Enforce channel policy in app code, not only on the client.</li>
                </ul>

                <h3>Scaling</h3>
                <ul>
                    <li>Run dedicated websocket-server instances behind a load balancer.</li>
                    <li>Plan Redis capacity for pub/sub and durable stream retention.</li>
                    <li>Tune send queue capacity to avoid slow-consumer memory spikes.</li>
                </ul>

                <h3>Failure handling</h3>
                <ul>
                    <li>Watch <code>/realtime/metrics.prom</code> for replay-gap and queue-drop trends.</li>
                    <li>If replay gaps spike, review retention settings and Redis memory pressure.</li>
                    <li>If slow-consumer disconnects spike, fix client consumption strategy first.</li>
                    <li>On <code>replay_gap</code>, clients should trigger a full HTTP state refresh.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/feature-realtime">Realtime / WebSocket</a> for the framework/app responsibility split.
                    </li>
                    <li>
                        <a href="#/cookbook/build-end-to-end-flow">Build an End-to-End Flow</a> for the combined API/jobs/realtime recipe.
                    </li>
                </ul>
            </div>
        </div>
    )
}
