use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, routing::get, Json, Router};
use bootstrap::boot::BootContext;
use core_realtime::{
    ws_handler, AllowAllPublishAuthorizer, AllowAllSubscribeAuthorizer, AuthResolver,
    ChannelPolicy, ChannelPolicyRegistry, PresenceManager, RealtimeMetricsSnapshot,
    RealtimeSubscriber, WsServerState,
};

pub async fn build_router(ctx: BootContext) -> anyhow::Result<Router> {
    let state = build_state(ctx).await?;

    Ok(Router::new()
        .route("/health", get(health))
        .route("/realtime/metrics", get(metrics))
        .route("/ws", get(ws_handler))
        .with_state(state))
}

pub async fn build_state(ctx: BootContext) -> anyhow::Result<WsServerState> {
    let presence = PresenceManager::new(
        &ctx.settings.redis.url,
        ctx.settings.realtime.presence_ttl_secs,
    )?;
    let subscriber = RealtimeSubscriber::new(&ctx.settings.redis.url)?;

    let state = WsServerState::new(
        ctx.settings.clone(),
        build_policy_registry(&ctx.settings),
        presence,
        subscriber,
        &ctx.settings.redis.url,
        build_auth_resolver(ctx.db.clone()),
    )?;
    state.spawn_pubsub_loop();
    Ok(state)
}

pub fn build_policy_registry(settings: &core_config::Settings) -> ChannelPolicyRegistry {
    ChannelPolicyRegistry::new(
        settings.realtime.enabled,
        channel_policies(&settings.realtime),
        Arc::new(AllowAllSubscribeAuthorizer),
    )
    .with_publish_authorizer(Arc::new(AllowAllPublishAuthorizer))
}

pub fn channel_policies(
    settings: &core_config::RealtimeSettings,
) -> HashMap<String, ChannelPolicy> {
    settings
        .channels
        .iter()
        .map(|(name, cfg)| {
            (
                name.to_ascii_lowercase(),
                ChannelPolicy {
                    enabled: cfg.enabled,
                    guard: cfg
                        .guard
                        .as_ref()
                        .map(|guard| guard.trim().to_string())
                        .filter(|guard| !guard.is_empty()),
                    presence_enabled: cfg.presence_enabled,
                },
            )
        })
        .collect()
}

pub fn build_auth_resolver(db: sqlx::PgPool) -> AuthResolver {
    Arc::new(move |token: String| {
        let db = db.clone();
        Box::pin(async move { generated::guards::authenticate_any_guard(&db, &token).await })
    })
}

async fn health() -> &'static str {
    "ok"
}

async fn metrics(State(state): State<WsServerState>) -> Json<RealtimeMetricsSnapshot> {
    Json(state.metrics_snapshot())
}
