import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter5WebsocketChannel() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 5: WebSocket Server + Channel Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Create a channel config, run websocket server, publish events, and subscribe
                    from client.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope and Defaults</h2>
                <ul>
                    <li>
                        Server process: <code>./bin/websocket-server</code> (starter binary
                        source: <code>app/src/bin/websocket-server.rs</code>).
                    </li>
                    <li>
                        WebSocket endpoint: <code>ws://127.0.0.1:3010/ws</code>.
                    </li>
                    <li>
                        Publish side can be API, CLI, or worker through{' '}
                        <code>core_realtime::RealtimePublisher</code>.
                    </li>
                </ul>

                <h2>Step 1: Add Channel Config</h2>
                <h3>
                    File: <code>app/configs.toml</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[realtime.channels.public_feed]
enabled = true
presence_enabled = false
max_messages_per_sec = 80`}</code>
                </pre>

                <h3>
                    File: <code>.env</code> (runtime)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`REALTIME_ENABLED=true
REALTIME_HOST=0.0.0.0
REALTIME_PORT=3010
REALTIME_REQUIRE_AUTH=false
REALTIME_DELIVERY_MODE=at_most_once`}</code>
                </pre>

                <h2>Step 2: Build WebSocket Router</h2>
                <h3>
                    File: <code>app/src/internal/realtime/mod.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::{routing::get, Router};
use core_realtime::{
    ws_handler, AuthResolver, ChannelPolicy, ChannelPolicyRegistry, PresenceManager,
    RealtimeSubscriber, WsServerState,
};
use std::{collections::HashMap, sync::Arc};

pub async fn build_router(ctx: bootstrap::boot::BootContext) -> anyhow::Result<Router> {
    let (subscribe_authorizer, publish_authorizer) = models::realtime_policy::authorizers();

    let policies = ctx
        .settings
        .realtime
        .channels
        .iter()
        .map(|(channel, cfg)| {
            (
                channel.to_string(),
                ChannelPolicy {
                    enabled: cfg.enabled,
                    guard: cfg.guard.clone(),
                    presence_enabled: cfg.presence_enabled,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let registry = ChannelPolicyRegistry::new(
        ctx.settings.realtime.enabled,
        policies,
        subscribe_authorizer,
    )
    .with_publish_authorizer(publish_authorizer);

    let presence = PresenceManager::new(
        &ctx.settings.redis.url,
        ctx.settings.realtime.presence_ttl_secs,
    )?;
    let subscriber = RealtimeSubscriber::new(&ctx.settings.redis.url)?;

    let db = ctx.db.clone();
    let auth_resolver: AuthResolver = Arc::new(move |token: String| {
        let db = db.clone();
        Box::pin(async move { models::generated::guards::authenticate_any_guard(&db, &token).await })
    });

    let ws_state = WsServerState::new(
        ctx.settings.clone(),
        registry,
        presence,
        subscriber,
        &ctx.settings.redis.url,
        auth_resolver,
    )?;
    ws_state.spawn_pubsub_loop();

    Ok(Router::new().route("/ws", get(ws_handler)).with_state(ws_state))
}`}</code>
                </pre>

                <h2>Step 3: Publish Event from API</h2>
                <h3>
                    File: <code>app/src/api/realtime.rs</code> (already provided)</h3>
                <p>
                    The existing API route <code>POST /admin/realtime/publish</code> can publish to
                    your new channel. It uses policy checks before publishing.
                </p>

                <h2>Step 4: Run Server and Subscribe</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# terminal 1: websocket server
./bin/websocket-server

# terminal 2: api server
./bin/api-server`}</code>
                </pre>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'subscribe', channel: 'public_feed' }));
};

ws.onmessage = (evt) => {
  console.log('ws message', JSON.parse(evt.data));
};`}</code>
                </pre>

                <h2>Step 5: Publish and Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`curl -X POST http://127.0.0.1:3000/api/v1/admin/realtime/publish \
  -H 'x-realtime-key: <APP_KEY>' \
  -H 'Content-Type: application/json' \
  -d '{
    "channel": "public_feed",
    "event": "article_published",
    "payload": {"id": 1, "title": "Hello"}
  }'`}</code>
                </pre>
                <p>
                    Expect client to receive <code>{`{"op":"event", ...}`}</code> payload for
                    <code>public_feed</code>.
                </p>
            </div>
        </div>
    )
}
