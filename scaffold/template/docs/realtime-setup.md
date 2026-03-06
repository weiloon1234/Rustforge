# Realtime Setup Guide

Use this when the generated starter needs realtime behavior beyond the shipped baseline.

## Current starter baseline

The starter already ships these pieces:

- `app/configs.toml` realtime settings and channel config
- `app/src/bin/websocket-server.rs` websocket server binary entrypoint
- `app/src/internal/realtime/mod.rs` app-owned realtime router/state builder
- framework runtime in `core-realtime`
- generated any-guard token resolution via `generated::guards::authenticate_any_guard(...)`

The shipped websocket server now exposes:

- `GET /health`
- `GET /realtime/metrics`
- `GET /ws` websocket upgrade route

It builds `WsServerState` from configured channels and uses default allow-all app authorizers on top of the framework's built-in channel-enabled, auth-required, and guard-match checks.

## Ownership split

- Framework owns:
  - websocket protocol
  - replay/presence/metrics runtime
  - `WsServerState`
  - `ChannelPolicyRegistry`
  - `RealtimePublisher`
- Starter/app owns:
  - channel names
  - subscribe/publish authorization rules beyond the framework baseline
  - which workflows or jobs publish events

## Step 1: Declare the channel in `app/configs.toml`

```toml
[realtime.channels.public]
enabled = true
guard = ""
presence_enabled = false

[realtime.channels.admin_notifications]
enabled = true
guard = "admin"
presence_enabled = true
```

Use `guard = "admin"` or another real guard when the channel should only accept that token type.

## Step 2: Understand the shipped policy baseline

The starter already maps `settings.realtime.channels` into `ChannelPolicy` and uses:

- `AllowAllSubscribeAuthorizer`
- `AllowAllPublishAuthorizer`
- `generated::guards::authenticate_any_guard(...)`

That means the framework already enforces:

- realtime globally enabled
- channel enabled
- auth required when configured
- channel guard match

If you need app-specific ability checks or room ownership rules, replace the default authorizers in `app/src/internal/realtime/mod.rs`.

## Step 3: Add custom authorizers in `app/src/internal/realtime/mod.rs`

```rust
use core_realtime::{
    ErrorCode, PolicyContext, PolicyDecision, PublishAuthorizer, SubscribeAuthorizer,
};

pub struct AdminSubscribeAuthorizer;
pub struct AdminPublishAuthorizer;

impl SubscribeAuthorizer for AdminSubscribeAuthorizer {
    fn authorize_subscribe(&self, context: &PolicyContext) -> PolicyDecision {
        if !context.has_ability("country.read") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "missing country.read");
        }

        if context.room() != Some("country:admin") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "unexpected room");
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
}
```

`PolicyContext` is the SSOT for websocket policy checks. Use:

- `context.guard()`
- `context.subject_id()`
- `context.room()`
- `context.has_ability(...)`

Do not invent a second permission matcher for websocket channels.

## Step 4: Replace the default registry when you need custom policy

The shipped starter already builds a real `WsServerState`. When you need custom subscribe/publish rules, replace the default allow-all registry with your own.

```rust
pub fn build_policy_registry(settings: &core_config::Settings) -> ChannelPolicyRegistry {
    let mut policies = HashMap::new();
    policies.insert(
        "admin_notifications".to_string(),
        ChannelPolicy {
            enabled: true,
            guard: Some("admin".to_string()),
            presence_enabled: true,
        },
    );

    ChannelPolicyRegistry::new(
        settings.realtime.enabled,
        policies,
        Arc::new(AdminSubscribeAuthorizer),
    )
    .with_publish_authorizer(Arc::new(AdminPublishAuthorizer))
}
```

Keep token resolution on the generated guard helper unless you truly need a different auth source.

## Step 5: Publish from workflows or jobs

Publish from the domain flow that owns the event.

```rust
use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};

#[derive(serde::Serialize)]
pub struct CountryStatusUpdated {
    pub iso2: String,
    pub status: String,
}

impl RealtimeEvent for CountryStatusUpdated {
    const CHANNEL: &'static str = "admin_notifications";
    const EVENT: &'static str = "country.status_updated";
}

let publisher = RealtimePublisher::from_realtime_settings(
    &state.settings.redis.url,
    &state.settings.realtime,
)?;

publisher
    .publish(
        RealtimeTarget::room("country:admin"),
        &CountryStatusUpdated {
            iso2: country.iso2.clone(),
            status: country.status.clone(),
        },
    )
    .await?;
```

If the event must be durable or retryable, publish from a queued job instead of directly in the request path.

## Step 6: Verify the wire flow

```bash
cargo check -p app
./bin/websocket-server

# then from a client:
# 1. connect ws://127.0.0.1:3010/ws
# 2. send {"op":"auth","token":"..."}
# 3. send {"op":"subscribe","channel":"admin_notifications","room":"country:admin"}
# 4. trigger the workflow or job that publishes the event
```

Check these paths explicitly:

- valid token + right guard + right ability => `auth_ok` then subscribe success
- wrong guard => forbidden
- missing ability => forbidden
- wrong room => forbidden

## Related docs

- Framework docs: realtime feature, websocket auth recipe, permission matrix
- Starter docs: `computed-model-values.md` when the event payload depends on view extensions
