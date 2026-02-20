export function RealtimeProtocolStateMachineFeature() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Realtime Protocol / State Machine
                </h1>
                <p className="text-xl text-gray-500">
                    Wire-level contract and production operations checklist for native WebSocket
                    realtime.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Protocol Contract</h2>
                <p>Client to server operations:</p>
                <ul>
                    <li>
                        <code>auth</code>: authenticate token payload
                    </li>
                    <li>
                        <code>subscribe</code>: subscribe by <code>channel</code> + optional <code>room</code>
                    </li>
                    <li>
                        <code>unsubscribe</code>: stop subscription
                    </li>
                    <li>
                        <code>ack</code>: acknowledge a <code>delivery_id</code> for checkpoint
                    </li>
                    <li>
                        <code>ping</code>: heartbeat
                    </li>
                </ul>
                <p>Server to client operations:</p>
                <ul>
                    <li>
                        <code>auth_ok</code>
                    </li>
                    <li>
                        <code>event</code>
                    </li>
                    <li>
                        <code>replay_gap</code>
                    </li>
                    <li>
                        <code>presence</code>
                    </li>
                    <li>
                        <code>ack_ok</code>
                    </li>
                    <li>
                        <code>error</code>
                    </li>
                    <li>
                        <code>pong</code>
                    </li>
                </ul>

                <h3>Event Envelope</h3>
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
                    Timestamp contract is <code>sent_at_unix_ms</code> only. Legacy
                    <code>sent_at_unix</code> is removed.
                </p>

                <h2>Connection State Machine</h2>
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

                <h2>Durable Replay Rules</h2>
                <ul>
                    <li>
                        Durable mode uses Redis Stream with <code>delivery_id</code>.
                    </li>
                    <li>
                        Client resume options:
                        <code>subscribe.since_id</code> (explicit cursor) or checkpoint from prior
                        <code>ack</code>.
                    </li>
                    <li>
                        If cursor is outside retained window, server emits <code>replay_gap</code>
                        then replays from earliest retained entries.
                    </li>
                    <li>
                        Retention is controlled by
                        <code>REALTIME_STREAM_MAX_LEN</code> and
                        <code>REALTIME_STREAM_RETENTION_SECS</code>.
                    </li>
                </ul>

                <h2>Production Checklist</h2>
                <h3>Security</h3>
                <ul>
                    <li>
                        Keep <code>REALTIME_REQUIRE_AUTH=true</code> in production.
                    </li>
                    <li>
                        Enforce channel guard and app policy checks for subscribe/publish.
                    </li>
                    <li>
                        Rotate PATs and keep refresh endpoint workflow app-defined.
                    </li>
                    <li>
                        Use TLS and avoid token in query string.
                    </li>
                </ul>

                <h3>Scaling</h3>
                <ul>
                    <li>
                        Run dedicated <code>websocket-server</code> instances behind LB (starter
                        binary: <code>./bin/websocket-server</code>).
                    </li>
                    <li>
                        Use Redis capacity planning for Pub/Sub + durable stream storage.
                    </li>
                    <li>
                        Tune <code>REALTIME_SEND_QUEUE_CAPACITY</code> to avoid memory spikes.
                    </li>
                    <li>
                        Watch connection/message rate limits and room cardinality.
                    </li>
                </ul>

                <h3>Incident Runbook</h3>
                <ul>
                    <li>
                        Check <code>/realtime/metrics.prom</code> for
                        replay-gap, queue-dropped, and slow-consumer trends.
                    </li>
                    <li>
                        If replay gaps spike, review stream retention controls and Redis memory.
                    </li>
                    <li>
                        If slow-consumer disconnects spike, adjust frontend consumption strategy and
                        queue/rate settings.
                    </li>
                    <li>
                        Use <code>cli realtime bench --json</code> in CI/perf jobs for regression
                        detection.
                    </li>
                </ul>

                <h3>Frontend Reconnect Strategy</h3>
                <ul>
                    <li>
                        Persist latest <code>delivery_id</code> locally.
                    </li>
                    <li>
                        Reconnect with exponential backoff and jitter.
                    </li>
                    <li>
                        On reconnect: <code>auth</code> then <code>subscribe</code> with
                        <code>since_id</code>.
                    </li>
                    <li>
                        If <code>replay_gap</code> arrives, trigger full-state refresh from HTTP API.
                    </li>
                </ul>
            </div>
        </div>
    )
}
