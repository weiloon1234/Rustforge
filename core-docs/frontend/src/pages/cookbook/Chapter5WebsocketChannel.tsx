import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter5WebsocketChannel() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Recipe: Add a Realtime Channel</h1>
                <p className="text-xl text-gray-500">
                    Add a channel by declaring runtime policy, building a policy registry, and
                    publishing events from workflows or jobs.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Add a new channel without inventing a second auth or event system. Channel
                    policy belongs to app code; websocket transport, replay, presence, and publish
                    plumbing belong to the framework.
                </p>

                <h2>Step 1: Declare the channel in config</h2>
                <p>
                    Channel enablement, guard matching, and presence support start in
                    <code> app/configs.toml</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[realtime.channels.public]
enabled = true
guard = ""
presence_enabled = false

[realtime.channels.admin_notifications]
enabled = true
guard = "admin"
presence_enabled = true`}</code>
                </pre>

                <h2>Step 2: Define subscribe and publish policy</h2>
                <p>
                    Use <code>PolicyContext</code> as the only authorization input. It already knows
                    the guard, subject, room, and raw abilities from the authenticated token.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::{
    ErrorCode, PolicyContext, PolicyDecision, PublishAuthorizer, SubscribeAuthorizer,
};

struct AdminSubscribeAuthorizer;
struct AdminPublishAuthorizer;

impl SubscribeAuthorizer for AdminSubscribeAuthorizer {
    fn authorize_subscribe(&self, context: &PolicyContext) -> PolicyDecision {
        if context.guard() != Some("admin") {
            return PolicyDecision::deny(ErrorCode::Unauthorized, "admin token required");
        }
        if !context.has_ability("country.read") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "missing country.read");
        }
        PolicyDecision::allow()
    }
}

impl PublishAuthorizer for AdminPublishAuthorizer {
    fn authorize_publish(&self, context: &PolicyContext) -> PolicyDecision {
        if !context.has_ability("country.manage") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "missing country.manage");
        }
        PolicyDecision::allow()
    }
}`}</code>
                </pre>

                <h2>Step 3: Build the websocket state</h2>
                <p>
                    The scaffold gives you the websocket binary and a placeholder
                    <code> app/src/internal/realtime/</code> module. The concrete registry/state
                    wiring is still app-owned and should look like this.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use std::collections::HashMap;
use std::sync::Arc;

use axum::{routing::get, Router};
use core_realtime::{
    ws_handler, AuthResolver, ChannelPolicy, ChannelPolicyRegistry, PresenceManager,
    RealtimeSubscriber, WsServerState,
};

let mut policies = HashMap::new();
policies.insert(
    "admin_notifications".to_string(),
    ChannelPolicy {
        enabled: true,
        guard: Some("admin".to_string()),
        presence_enabled: true,
    },
);

let registry = ChannelPolicyRegistry::new(
    true,
    policies,
    Arc::new(AdminSubscribeAuthorizer),
).with_publish_authorizer(Arc::new(AdminPublishAuthorizer));

let presence = PresenceManager::new(&ctx.settings.redis.url, ctx.settings.realtime.presence_ttl_secs)?;
let subscriber = RealtimeSubscriber::new(&ctx.settings.redis.url)?;
let auth_resolver: AuthResolver = Arc::new(|token| Box::pin(resolve_admin_identity(token)));
let state = WsServerState::new(
    Arc::new(ctx.settings.clone()),
    registry,
    presence,
    subscriber,
    &ctx.settings.redis.url,
    auth_resolver,
)?;
state.spawn_pubsub_loop();

let app = Router::new().route("/ws", get(ws_handler)).with_state(state);`}</code>
                </pre>

                <h2>Step 4: Publish from workflow or job</h2>
                <p>
                    The event type is app-defined. The publisher is framework-owned.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};

#[derive(serde::Serialize)]
struct CountryStatusUpdated {
    iso2: String,
    status: String,
}

impl RealtimeEvent for CountryStatusUpdated {
    const CHANNEL: &'static str = "admin_notifications";
    const EVENT: &'static str = "country.status_updated";
}

let publisher = RealtimePublisher::from_realtime_settings(
    &state.settings.redis.url,
    &state.settings.realtime,
)?;

publisher.publish(
    RealtimeTarget::room("country:admin"),
    &CountryStatusUpdated { iso2, status },
).await?;`}</code>
                </pre>

                <h2>Step 5: Verify with the actual wire flow</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/websocket-server
# connect websocket client
# auth with an admin token
# subscribe to admin_notifications + room
# trigger the workflow/job publish
# confirm event frame arrives`}</code>
                </pre>

                <h2>Practical rule</h2>
                <p>
                    Channel names and authorization rules are app-owned. Replay, presence, and
                    transport protocol stay framework-owned.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/feature-realtime">Realtime / WebSocket</a></li>
                    <li><a href="#/feature-realtime-protocol">Realtime Protocol &amp; Runbook</a></li>
                    <li><a href="#/cookbook/add-websocket-auth">Add WebSocket Auth</a></li>
                    <li><a href="#/jobs">Job Queue</a> when publish should be durable.</li>
                </ul>
            </div>
        </div>
    )
}
