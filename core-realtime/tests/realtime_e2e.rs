use anyhow::Context;
use axum::{routing::get, Router};
use core_config::{
    AppSettings, AuthSettings, CdnSettings, CorsSettings, DbSettings, GuardConfig,
    HttpLogSettings, MailSettings, MiddlewareSettings, RealtimeChannelConfig,
    RealtimeDeliveryMode, RealtimeSettings, RedisSettings, S3Settings, ServerSettings, Settings,
    WorkerSettings,
};
use core_realtime::{
    ws_handler, AllowAllSubscribeAuthorizer, AuthResolver, ChannelPolicy, ChannelPolicyRegistry,
    PresenceManager, RealtimePublishSettings, RealtimePublisher, RealtimeSubscriber, WsServerState,
};
use core_web::auth::AuthIdentity;
use futures_util::{SinkExt, StreamExt};
use redis::AsyncCommands;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn durable_replay_ack_checkpoint_reconnect_flow() -> anyhow::Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
    if !redis_available(&redis_url).await {
        eprintln!(
            "skipping realtime e2e test: redis is not available at {}",
            redis_url
        );
        return Ok(());
    }

    let channel = format!("it_{}", Uuid::new_v4().simple());
    let room = format!("room_{}", Uuid::new_v4().simple());
    let settings = test_settings(&redis_url, &channel, 2_048, 100);

    let mut channel_policies = HashMap::new();
    channel_policies.insert(
        channel.clone(),
        ChannelPolicy {
            enabled: true,
            guard: Some("web".to_string()),
            presence_enabled: true,
        },
    );
    let policy_registry = ChannelPolicyRegistry::new(
        true,
        channel_policies,
        Arc::new(AllowAllSubscribeAuthorizer),
    );

    let presence = PresenceManager::new(&redis_url, settings.realtime.presence_ttl_secs)?;
    let subscriber = RealtimeSubscriber::new(&redis_url)?;
    let auth_resolver: AuthResolver = Arc::new(|token: String| {
        Box::pin(async move {
            if token == "good-token" {
                Some(AuthIdentity {
                    subject_id: "subject-1".to_string(),
                    guard: "web".to_string(),
                    abilities: vec!["*".to_string()],
                    token_id: None,
                })
            } else {
                None
            }
        })
    });

    let state = WsServerState::new(
        settings.clone(),
        policy_registry,
        presence,
        subscriber,
        &redis_url,
        auth_resolver,
    )?;
    state.spawn_pubsub_loop();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    let ws_url = format!("ws://{}/ws", addr);
    let publisher = RealtimePublisher::new_with_settings(
        &redis_url,
        RealtimePublishSettings {
            delivery_mode: RealtimeDeliveryMode::Durable,
            stream_max_len: 2048,
            stream_retention_secs: 0,
        },
    )?;

    let (mut ws1, _) = connect_async(&ws_url).await?;
    send_client_message(&mut ws1, json!({ "op": "auth", "token": "good-token" })).await?;
    let _ = recv_until_op(&mut ws1, "auth_ok").await?;
    send_client_message(
        &mut ws1,
        json!({
            "op": "subscribe",
            "channel": channel,
            "room": room,
        }),
    )
    .await?;

    publisher
        .publish_raw(
            &channel,
            "it_event",
            Some(&room),
            json!({ "seq": 1, "kind": "live" }),
        )
        .await?;

    let event1 = recv_until_op(&mut ws1, "event").await?;
    assert_eq!(event1["payload"]["seq"], 1);
    let delivery_id_1 = event1["delivery_id"]
        .as_str()
        .context("event1 should include delivery_id")?
        .to_string();

    send_client_message(
        &mut ws1,
        json!({
            "op": "ack",
            "channel": channel,
            "room": room,
            "delivery_id": delivery_id_1,
        }),
    )
    .await?;
    let ack1 = recv_until_op(&mut ws1, "ack_ok").await?;
    assert_eq!(ack1["delivery_id"], event1["delivery_id"]);
    let checkpoint_after_ack1 =
        load_checkpoint(&redis_url, "subject-1", &channel, Some(&room)).await?;
    assert_eq!(
        checkpoint_after_ack1.as_deref(),
        Some(delivery_id_1.as_str())
    );
    ws1.close(None).await?;

    publisher
        .publish_raw(
            &channel,
            "it_event",
            Some(&room),
            json!({ "seq": 2, "kind": "replay" }),
        )
        .await?;

    let (mut ws2, _) = connect_async(&ws_url).await?;
    send_client_message(&mut ws2, json!({ "op": "auth", "token": "good-token" })).await?;
    let _ = recv_until_op(&mut ws2, "auth_ok").await?;
    send_client_message(
        &mut ws2,
        json!({
            "op": "subscribe",
            "channel": channel,
            "room": room,
        }),
    )
    .await?;

    let replay_event = recv_until_op(&mut ws2, "event").await?;
    assert_eq!(replay_event["payload"]["seq"], 2);
    assert_eq!(replay_event["payload"]["kind"], "replay");
    let delivery_id_2 = replay_event["delivery_id"]
        .as_str()
        .context("replay event should include delivery_id")?
        .to_string();

    send_client_message(
        &mut ws2,
        json!({
            "op": "ack",
            "channel": channel,
            "room": room,
            "delivery_id": delivery_id_2,
        }),
    )
    .await?;
    let ack2 = recv_until_op(&mut ws2, "ack_ok").await?;
    assert_eq!(ack2["delivery_id"], replay_event["delivery_id"]);
    let checkpoint_after_ack2 =
        load_checkpoint(&redis_url, "subject-1", &channel, Some(&room)).await?;
    assert_eq!(
        checkpoint_after_ack2.as_deref(),
        Some(delivery_id_2.as_str())
    );
    ws2.close(None).await?;

    server.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn replay_gap_emits_signal_and_updates_metrics() -> anyhow::Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
    if !redis_available(&redis_url).await {
        eprintln!(
            "skipping realtime replay-gap test: redis is not available at {}",
            redis_url
        );
        return Ok(());
    }

    let channel = format!("it_{}", Uuid::new_v4().simple());
    let room = format!("room_{}", Uuid::new_v4().simple());
    let settings = test_settings(&redis_url, &channel, 2, 1);

    let mut channel_policies = HashMap::new();
    channel_policies.insert(
        channel.clone(),
        ChannelPolicy {
            enabled: true,
            guard: Some("web".to_string()),
            presence_enabled: true,
        },
    );
    let policy_registry = ChannelPolicyRegistry::new(
        true,
        channel_policies,
        Arc::new(AllowAllSubscribeAuthorizer),
    );

    let presence = PresenceManager::new(&redis_url, settings.realtime.presence_ttl_secs)?;
    let subscriber = RealtimeSubscriber::new(&redis_url)?;
    let auth_resolver: AuthResolver = Arc::new(|token: String| {
        Box::pin(async move {
            if token == "good-token" {
                Some(AuthIdentity {
                    subject_id: "subject-1".to_string(),
                    guard: "web".to_string(),
                    abilities: vec!["*".to_string()],
                    token_id: None,
                })
            } else {
                None
            }
        })
    });

    let state = WsServerState::new(
        settings.clone(),
        policy_registry,
        presence,
        subscriber,
        &redis_url,
        auth_resolver,
    )?;
    state.spawn_pubsub_loop();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    let ws_url = format!("ws://{}/ws", addr);
    let publisher = RealtimePublisher::new_with_settings(
        &redis_url,
        RealtimePublishSettings {
            delivery_mode: RealtimeDeliveryMode::Durable,
            stream_max_len: 2,
            stream_retention_secs: 0,
        },
    )?;

    let (mut ws1, _) = connect_async(&ws_url).await?;
    send_client_message(&mut ws1, json!({ "op": "auth", "token": "good-token" })).await?;
    let _ = recv_until_op(&mut ws1, "auth_ok").await?;
    send_client_message(
        &mut ws1,
        json!({
            "op": "subscribe",
            "channel": channel,
            "room": room,
        }),
    )
    .await?;

    publisher
        .publish_raw(&channel, "it_event", Some(&room), json!({ "seq": 1 }))
        .await?;
    let first = recv_until_op(&mut ws1, "event").await?;
    let stale_delivery_id = first["delivery_id"]
        .as_str()
        .context("expected first event delivery id")?
        .to_string();
    ws1.close(None).await?;

    for seq in 2..=8 {
        publisher
            .publish_raw(&channel, "it_event", Some(&room), json!({ "seq": seq }))
            .await?;
    }
    // Keep retention deterministic in test; publisher uses approximate trim for production throughput.
    trim_stream_maxlen_exact(&redis_url, &channel, 2).await?;
    assert!(
        !stream_id_exists(&redis_url, &channel, &stale_delivery_id).await?,
        "expected stale delivery id to be evicted before replay-gap assertion"
    );

    let (mut ws2, _) = connect_async(&ws_url).await?;
    send_client_message(&mut ws2, json!({ "op": "auth", "token": "good-token" })).await?;
    let _ = recv_until_op(&mut ws2, "auth_ok").await?;
    send_client_message(
        &mut ws2,
        json!({
            "op": "subscribe",
            "channel": channel,
            "room": room,
            "since_id": stale_delivery_id,
            "replay_limit": 20,
        }),
    )
    .await?;

    let replay_gap = recv_until_op(&mut ws2, "replay_gap").await?;
    assert_eq!(replay_gap["channel"], channel);
    assert_eq!(replay_gap["room"], room);

    // Server resumes replay from earliest retained window after replay_gap.
    let replayed = recv_until_op(&mut ws2, "event").await?;
    assert_eq!(replayed["channel"], channel);
    assert_eq!(replayed["room"], room);
    ws2.close(None).await?;

    tokio::time::sleep(Duration::from_millis(150)).await;
    let metrics = state.metrics_snapshot();
    assert!(
        metrics.replay_gap_total >= 1,
        "expected replay_gap_total>=1, got {}",
        metrics.replay_gap_total
    );
    assert!(
        metrics.replay_gap_alerts >= 1,
        "expected replay_gap_alerts>=1, got {}",
        metrics.replay_gap_alerts
    );
    let scoped = metrics
        .scopes
        .iter()
        .find(|s| s.channel == channel && s.room.as_deref() == Some(room.as_str()))
        .context("expected scope metrics for channel+room")?;
    assert!(
        scoped.replay_gap >= 1,
        "expected scoped replay gap >= 1, got {}",
        scoped.replay_gap
    );

    server.abort();
    Ok(())
}

async fn send_client_message(ws: &mut WsStream, payload: Value) -> anyhow::Result<()> {
    ws.send(Message::Text(payload.to_string().into())).await?;
    Ok(())
}

async fn recv_until_op(ws: &mut WsStream, op: &str) -> anyhow::Result<Value> {
    let mut attempts = 0usize;
    while attempts < 40 {
        attempts += 1;
        let next = timeout(Duration::from_secs(5), ws.next())
            .await
            .context("websocket receive timeout")?;
        let frame = next.context("websocket stream closed")??;
        let Message::Text(text) = frame else {
            continue;
        };
        let value: Value = serde_json::from_str(&text)?;
        let current_op = value
            .get("op")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if current_op == op.to_ascii_lowercase() {
            return Ok(value);
        }
    }
    anyhow::bail!("did not receive op='{}' within attempt budget", op);
}

fn test_settings(
    redis_url: &str,
    channel: &str,
    stream_max_len: usize,
    replay_gap_alert_threshold: u64,
) -> Arc<Settings> {
    let mut channels = HashMap::new();
    channels.insert(
        channel.to_string(),
        RealtimeChannelConfig {
            enabled: true,
            guard: Some("web".to_string()),
            presence_enabled: true,
            max_message_bytes: None,
            max_frame_bytes: None,
            max_messages_per_sec: None,
        },
    );

    let mut guards = HashMap::new();
    guards.insert(
        "web".to_string(),
        GuardConfig {
            provider: "users".to_string(),
            ttl_min: 60,
            refresh_ttl_days: 30,
        },
    );

    fn app_settings() -> AppSettings {
        AppSettings {
            name: "realtime-test".into(),
            env: "test".into(),
            key: "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            ..AppSettings::default()
        }
    }

    fn cors_settings() -> CorsSettings {
        CorsSettings::default()
    }

    Arc::new(Settings {
        app: app_settings(),
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 0,
        },
        realtime: RealtimeSettings {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 0,
            heartbeat_secs: 5,
            presence_ttl_secs: 30,
            max_connections: 1_000,
            max_message_bytes: 64 * 1024,
            max_frame_bytes: 64 * 1024,
            max_messages_per_sec: 300,
            send_queue_capacity: 64,
            require_auth: true,
            checkpoint_enabled: true,
            checkpoint_ttl_secs: 3_600,
            delivery_mode: RealtimeDeliveryMode::Durable,
            stream_max_len,
            stream_retention_secs: 0,
            replay_limit_default: 200,
            replay_limit_max: 1_000,
            replay_gap_alert_threshold,
            replay_gap_alert_window_secs: 60,
            channels,
        },
        db: DbSettings {
            url: "postgres://localhost/test".to_string(),
            max_connections: 1,
            connect_timeout: Duration::from_secs(1),
            snowflake_node_id: 1,
            sql_profiler_enabled: false,
            sql_profiler_retention_days: 7,
        },
        redis: RedisSettings {
            url: redis_url.to_string(),
            prefix: None,
        },
        s3: S3Settings::default(),
        cdn: CdnSettings { base_url: None },
        worker: WorkerSettings {
            enabled: false,
            concurrency: 1,
            sweep_interval: 30,
        },
        i18n: core_i18n::config::I18nSettings {
            default_locale: "en",
            supported_locales: &["en"],
            default_timezone: core_i18n::config::I18nSettings::parse_utc_offset("+00:00")
                .expect("valid timezone"),
            default_timezone_str: "+00:00".to_string(),
        },
        middleware: MiddlewareSettings {
            rate_limit_per_second: 100,
            rate_limit_burst: 100,
            timeout_secs: 30,
            body_limit_mb: 10,
        },
        auth: AuthSettings {
            default_guard: "web".to_string(),
            guards,
        },
        mail: MailSettings::default(),
        http_log: HttpLogSettings {
            webhook_enabled: false,
            webhook_paths: String::new(),
            client_enabled: false,
            retention_days: 7,
        },
        cors: cors_settings(),
        tree: Arc::new(toml::Value::Table(toml::map::Map::new())),
    })
}

async fn redis_available(redis_url: &str) -> bool {
    let Ok(client) = redis::Client::open(redis_url) else {
        return false;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return false;
    };
    conn.ping::<String>().await.is_ok()
}

async fn load_checkpoint(
    redis_url: &str,
    subject_id: &str,
    channel: &str,
    room: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let room_key = room.map(str::trim).filter(|v| !v.is_empty()).unwrap_or("_");
    let key = format!("rt:checkpoint:{subject_id}:{channel}:{room_key}");
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let checkpoint: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
    Ok(checkpoint.filter(|v| !v.trim().is_empty()))
}

async fn trim_stream_maxlen_exact(
    redis_url: &str,
    channel: &str,
    max_len: usize,
) -> anyhow::Result<()> {
    let key = format!("rt:stream:{channel}");
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: usize = redis::cmd("XTRIM")
        .arg(key)
        .arg("MAXLEN")
        .arg(max_len.max(1))
        .query_async(&mut conn)
        .await?;
    Ok(())
}

async fn stream_id_exists(redis_url: &str, channel: &str, stream_id: &str) -> anyhow::Result<bool> {
    let key = format!("rt:stream:{channel}");
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let entries: Vec<(String, Vec<(String, String)>)> = redis::cmd("XRANGE")
        .arg(key)
        .arg(stream_id)
        .arg(stream_id)
        .arg("COUNT")
        .arg(1)
        .query_async(&mut conn)
        .await?;
    Ok(!entries.is_empty())
}
