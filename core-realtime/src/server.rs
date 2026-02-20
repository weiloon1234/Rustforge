use crate::auth::ConnectionAuthState;
use crate::presence::PresenceManager;
use crate::pubsub::RealtimeSubscriber;
use crate::registry::{
    ChannelPolicyRegistry, PolicyAction, PolicyContext, PolicyMetadata, PolicySource,
};
use crate::types::{AuthIdentityPublic, ClientMessage, ErrorCode, PubSubEnvelope, ServerMessage};
use crate::AuthResolver;
use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use core_config::RealtimeDeliveryMode;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use time::OffsetDateTime;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::time::{sleep, timeout, Duration};
use tracing::warn;
use uuid::Uuid;

const AUTH_TIMEOUT_MULTIPLIER: u64 = 2;
const READ_TIMEOUT_MULTIPLIER: u64 = 3;
const RATE_WINDOW_SECS: u64 = 1;
const LATENCY_BUCKETS_MS: [u64; 9] = [1, 5, 10, 25, 50, 100, 250, 500, 1000];

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConnectionCloseSignal {
    None,
    SlowConsumer,
}

impl Default for ConnectionCloseSignal {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone)]
struct ConnectionHandle {
    sender: mpsc::Sender<ServerMessage>,
    close_tx: watch::Sender<ConnectionCloseSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SubscriptionKey {
    channel: String,
    room: Option<String>,
}

#[derive(Default)]
struct HubState {
    senders: HashMap<Uuid, ConnectionHandle>,
    subscriptions: HashMap<SubscriptionKey, HashSet<Uuid>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RealtimeChannelMetricsSnapshot {
    pub channel: String,
    pub subscribe_success: u64,
    pub subscribe_denied: u64,
    pub events_live_dispatched: u64,
    pub events_replayed: u64,
    pub messages_out: u64,
}

#[derive(Debug, Clone, Default)]
struct RealtimeChannelMetrics {
    subscribe_success: u64,
    subscribe_denied: u64,
    events_live_dispatched: u64,
    events_replayed: u64,
    messages_out: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeScopeMetricsSnapshot {
    pub channel: String,
    pub room: Option<String>,
    pub events_live_dispatched: u64,
    pub events_replayed: u64,
    pub replay_gap: u64,
    pub e2e_latency_ms: RealtimeHistogramSnapshot,
}

#[derive(Debug, Clone)]
struct RealtimeScopeMetrics {
    events_live_dispatched: u64,
    events_replayed: u64,
    replay_gap: u64,
    e2e_latency_ms: RealtimeHistogram,
}

impl Default for RealtimeScopeMetrics {
    fn default() -> Self {
        Self {
            events_live_dispatched: 0,
            events_replayed: 0,
            replay_gap: 0,
            e2e_latency_ms: RealtimeHistogram::with_buckets(&LATENCY_BUCKETS_MS),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RealtimeScopeKey {
    channel: String,
    room: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RealtimeReplayGapSloSnapshot {
    pub window_count: u64,
    pub window_started_unix: i64,
    pub alert_triggered: bool,
    pub threshold: u64,
    pub window_secs: u64,
}

#[derive(Debug, Clone, Default)]
struct ReplayGapWindowState {
    window_count: u64,
    window_started_unix: i64,
    alert_triggered: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeHistogramSnapshot {
    pub buckets_ms: Vec<u64>,
    pub bucket_counts: Vec<u64>,
    pub count: u64,
    pub sum_ms: u64,
}

#[derive(Debug, Clone)]
struct RealtimeHistogram {
    buckets_ms: Vec<u64>,
    bucket_counts: Vec<u64>,
    count: u64,
    sum_ms: u64,
}

impl RealtimeHistogram {
    fn with_buckets(buckets_ms: &[u64]) -> Self {
        let mut normalized = buckets_ms.to_vec();
        normalized.sort_unstable();
        normalized.dedup();
        Self {
            bucket_counts: vec![0; normalized.len() + 1],
            buckets_ms: normalized,
            count: 0,
            sum_ms: 0,
        }
    }

    fn observe(&mut self, sample_ms: u64) {
        self.observe_n(sample_ms, 1);
    }

    fn observe_n(&mut self, sample_ms: u64, sample_count: u64) {
        if sample_count == 0 {
            return;
        }
        let idx = self
            .buckets_ms
            .iter()
            .position(|bound| sample_ms <= *bound)
            .unwrap_or(self.buckets_ms.len());
        self.bucket_counts[idx] = self.bucket_counts[idx].saturating_add(sample_count);
        self.count = self.count.saturating_add(sample_count);
        self.sum_ms = self
            .sum_ms
            .saturating_add(sample_ms.saturating_mul(sample_count));
    }

    fn snapshot(&self) -> RealtimeHistogramSnapshot {
        RealtimeHistogramSnapshot {
            buckets_ms: self.buckets_ms.clone(),
            bucket_counts: self.bucket_counts.clone(),
            count: self.count,
            sum_ms: self.sum_ms,
        }
    }
}

struct RealtimeMetrics {
    connections_opened: AtomicU64,
    connections_closed: AtomicU64,
    current_connections: AtomicU64,
    auth_success: AtomicU64,
    auth_failure: AtomicU64,
    subscribe_success: AtomicU64,
    subscribe_denied: AtomicU64,
    messages_in: AtomicU64,
    messages_out: AtomicU64,
    events_live_dispatched: AtomicU64,
    events_replayed: AtomicU64,
    replay_gap_total: AtomicU64,
    replay_gap_alerts: AtomicU64,
    outbound_queue_dropped: AtomicU64,
    slow_consumer_disconnects: AtomicU64,
    client_op_auth: AtomicU64,
    client_op_subscribe: AtomicU64,
    client_op_unsubscribe: AtomicU64,
    client_op_ack: AtomicU64,
    client_op_ping: AtomicU64,
    invalid_messages: AtomicU64,
    errors_unauthorized: AtomicU64,
    errors_channel_disabled: AtomicU64,
    errors_forbidden: AtomicU64,
    errors_invalid_message: AtomicU64,
    errors_rate_limited: AtomicU64,
    per_channel: Mutex<HashMap<String, RealtimeChannelMetrics>>,
    per_scope: Mutex<HashMap<RealtimeScopeKey, RealtimeScopeMetrics>>,
    replay_gap_window: Mutex<ReplayGapWindowState>,
    inbound_process_latency_ms: Mutex<RealtimeHistogram>,
    pubsub_dispatch_latency_ms: Mutex<RealtimeHistogram>,
}

impl Default for RealtimeMetrics {
    fn default() -> Self {
        Self {
            connections_opened: AtomicU64::default(),
            connections_closed: AtomicU64::default(),
            current_connections: AtomicU64::default(),
            auth_success: AtomicU64::default(),
            auth_failure: AtomicU64::default(),
            subscribe_success: AtomicU64::default(),
            subscribe_denied: AtomicU64::default(),
            messages_in: AtomicU64::default(),
            messages_out: AtomicU64::default(),
            events_live_dispatched: AtomicU64::default(),
            events_replayed: AtomicU64::default(),
            replay_gap_total: AtomicU64::default(),
            replay_gap_alerts: AtomicU64::default(),
            outbound_queue_dropped: AtomicU64::default(),
            slow_consumer_disconnects: AtomicU64::default(),
            client_op_auth: AtomicU64::default(),
            client_op_subscribe: AtomicU64::default(),
            client_op_unsubscribe: AtomicU64::default(),
            client_op_ack: AtomicU64::default(),
            client_op_ping: AtomicU64::default(),
            invalid_messages: AtomicU64::default(),
            errors_unauthorized: AtomicU64::default(),
            errors_channel_disabled: AtomicU64::default(),
            errors_forbidden: AtomicU64::default(),
            errors_invalid_message: AtomicU64::default(),
            errors_rate_limited: AtomicU64::default(),
            per_channel: Mutex::new(HashMap::new()),
            per_scope: Mutex::new(HashMap::new()),
            replay_gap_window: Mutex::new(ReplayGapWindowState::default()),
            inbound_process_latency_ms: Mutex::new(RealtimeHistogram::with_buckets(
                &LATENCY_BUCKETS_MS,
            )),
            pubsub_dispatch_latency_ms: Mutex::new(RealtimeHistogram::with_buckets(
                &LATENCY_BUCKETS_MS,
            )),
        }
    }
}

impl RealtimeMetrics {
    fn incr_op_auth(&self) {
        self.client_op_auth.fetch_add(1, Ordering::Relaxed);
    }

    fn incr_op_subscribe(&self) {
        self.client_op_subscribe.fetch_add(1, Ordering::Relaxed);
    }

    fn incr_op_unsubscribe(&self) {
        self.client_op_unsubscribe.fetch_add(1, Ordering::Relaxed);
    }

    fn incr_op_ack(&self) {
        self.client_op_ack.fetch_add(1, Ordering::Relaxed);
    }

    fn incr_op_ping(&self) {
        self.client_op_ping.fetch_add(1, Ordering::Relaxed);
    }

    fn incr_invalid_message(&self) {
        self.invalid_messages.fetch_add(1, Ordering::Relaxed);
    }

    fn observe_error_code(&self, code: ErrorCode) {
        match code {
            ErrorCode::Unauthorized => {
                self.errors_unauthorized.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCode::ChannelDisabled => {
                self.errors_channel_disabled.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCode::Forbidden => {
                self.errors_forbidden.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCode::InvalidMessage => {
                self.errors_invalid_message.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCode::RateLimited => {
                self.errors_rate_limited.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn observe_inbound_process_latency(&self, elapsed_ms: u64) {
        if let Ok(mut histogram) = self.inbound_process_latency_ms.lock() {
            histogram.observe(elapsed_ms);
        }
    }

    fn observe_pubsub_dispatch_latency(&self, elapsed_ms: u64) {
        if let Ok(mut histogram) = self.pubsub_dispatch_latency_ms.lock() {
            histogram.observe(elapsed_ms);
        }
    }

    fn incr_subscribe_success_channel(&self, channel: &str) {
        if let Ok(mut channels) = self.per_channel.lock() {
            let entry = channels.entry(channel.to_string()).or_default();
            entry.subscribe_success = entry.subscribe_success.saturating_add(1);
        }
    }

    fn incr_subscribe_denied_channel(&self, channel: &str) {
        if let Ok(mut channels) = self.per_channel.lock() {
            let entry = channels.entry(channel.to_string()).or_default();
            entry.subscribe_denied = entry.subscribe_denied.saturating_add(1);
        }
    }

    fn add_events_live_dispatched_channel(&self, channel: &str, delivered: u64) {
        if delivered == 0 {
            return;
        }
        if let Ok(mut channels) = self.per_channel.lock() {
            let entry = channels.entry(channel.to_string()).or_default();
            entry.events_live_dispatched = entry.events_live_dispatched.saturating_add(delivered);
            entry.messages_out = entry.messages_out.saturating_add(delivered);
        }
    }

    fn add_events_replayed_channel(&self, channel: &str, delivered: u64) {
        if delivered == 0 {
            return;
        }
        if let Ok(mut channels) = self.per_channel.lock() {
            let entry = channels.entry(channel.to_string()).or_default();
            entry.events_replayed = entry.events_replayed.saturating_add(delivered);
            entry.messages_out = entry.messages_out.saturating_add(delivered);
        }
    }

    fn add_messages_out_channel(&self, channel: &str, delivered: u64) {
        if delivered == 0 {
            return;
        }
        if let Ok(mut channels) = self.per_channel.lock() {
            let entry = channels.entry(channel.to_string()).or_default();
            entry.messages_out = entry.messages_out.saturating_add(delivered);
        }
    }

    fn add_scope_events_live_dispatched(
        &self,
        channel: &str,
        room: Option<&str>,
        delivered: u64,
        e2e_latency_ms: Option<u64>,
    ) {
        if delivered == 0 {
            return;
        }
        if let Ok(mut scopes) = self.per_scope.lock() {
            let entry = scopes
                .entry(RealtimeScopeKey {
                    channel: channel.to_string(),
                    room: room.map(ToString::to_string),
                })
                .or_default();
            entry.events_live_dispatched = entry.events_live_dispatched.saturating_add(delivered);
            if let Some(latency) = e2e_latency_ms {
                entry.e2e_latency_ms.observe_n(latency, delivered);
            }
        }
    }

    fn add_scope_events_replayed(
        &self,
        channel: &str,
        room: Option<&str>,
        delivered: u64,
        e2e_latency_ms: Option<u64>,
    ) {
        if delivered == 0 {
            return;
        }
        if let Ok(mut scopes) = self.per_scope.lock() {
            let entry = scopes
                .entry(RealtimeScopeKey {
                    channel: channel.to_string(),
                    room: room.map(ToString::to_string),
                })
                .or_default();
            entry.events_replayed = entry.events_replayed.saturating_add(delivered);
            if let Some(latency) = e2e_latency_ms {
                entry.e2e_latency_ms.observe_n(latency, delivered);
            }
        }
    }

    fn record_replay_gap(
        &self,
        channel: &str,
        room: Option<&str>,
        replay_gap_alert_threshold: u64,
        replay_gap_alert_window_secs: u64,
    ) {
        self.replay_gap_total.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut scopes) = self.per_scope.lock() {
            let entry = scopes
                .entry(RealtimeScopeKey {
                    channel: channel.to_string(),
                    room: room.map(ToString::to_string),
                })
                .or_default();
            entry.replay_gap = entry.replay_gap.saturating_add(1);
        }

        let threshold = replay_gap_alert_threshold;
        if threshold == 0 {
            return;
        }
        let window_secs = replay_gap_alert_window_secs.max(1);
        if let Ok(mut window) = self.replay_gap_window.lock() {
            let now_unix = OffsetDateTime::now_utc().unix_timestamp();
            if window.window_started_unix == 0
                || now_unix.saturating_sub(window.window_started_unix) >= window_secs as i64
            {
                window.window_started_unix = now_unix;
                window.window_count = 0;
                window.alert_triggered = false;
            }
            window.window_count = window.window_count.saturating_add(1);
            if !window.alert_triggered && window.window_count >= threshold {
                window.alert_triggered = true;
                self.replay_gap_alerts.fetch_add(1, Ordering::Relaxed);
                warn!(
                    channel = channel,
                    room = room.unwrap_or("_"),
                    replay_gap_window_count = window.window_count,
                    replay_gap_threshold = threshold,
                    replay_gap_window_secs = window_secs,
                    "realtime replay gap slo threshold reached"
                );
            }
        }
    }

    fn channels_snapshot(&self) -> Vec<RealtimeChannelMetricsSnapshot> {
        let mut channels = if let Ok(guard) = self.per_channel.lock() {
            guard
                .iter()
                .map(|(channel, value)| RealtimeChannelMetricsSnapshot {
                    channel: channel.clone(),
                    subscribe_success: value.subscribe_success,
                    subscribe_denied: value.subscribe_denied,
                    events_live_dispatched: value.events_live_dispatched,
                    events_replayed: value.events_replayed,
                    messages_out: value.messages_out,
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        channels.sort_by(|a, b| a.channel.cmp(&b.channel));
        channels
    }

    fn scopes_snapshot(&self) -> Vec<RealtimeScopeMetricsSnapshot> {
        let mut scopes = if let Ok(guard) = self.per_scope.lock() {
            guard
                .iter()
                .map(|(scope, value)| RealtimeScopeMetricsSnapshot {
                    channel: scope.channel.clone(),
                    room: scope.room.clone(),
                    events_live_dispatched: value.events_live_dispatched,
                    events_replayed: value.events_replayed,
                    replay_gap: value.replay_gap,
                    e2e_latency_ms: value.e2e_latency_ms.snapshot(),
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        scopes.sort_by(|a, b| {
            a.channel.cmp(&b.channel).then_with(|| {
                a.room
                    .as_deref()
                    .unwrap_or("_")
                    .cmp(b.room.as_deref().unwrap_or("_"))
            })
        });
        scopes
    }

    fn replay_gap_slo_snapshot(
        &self,
        replay_gap_alert_threshold: u64,
        replay_gap_alert_window_secs: u64,
    ) -> RealtimeReplayGapSloSnapshot {
        if let Ok(window) = self.replay_gap_window.lock() {
            RealtimeReplayGapSloSnapshot {
                window_count: window.window_count,
                window_started_unix: window.window_started_unix,
                alert_triggered: window.alert_triggered,
                threshold: replay_gap_alert_threshold,
                window_secs: replay_gap_alert_window_secs.max(1),
            }
        } else {
            RealtimeReplayGapSloSnapshot {
                threshold: replay_gap_alert_threshold,
                window_secs: replay_gap_alert_window_secs.max(1),
                ..RealtimeReplayGapSloSnapshot::default()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeMetricsSnapshot {
    pub connections_opened: u64,
    pub connections_closed: u64,
    pub current_connections: u64,
    pub auth_success: u64,
    pub auth_failure: u64,
    pub subscribe_success: u64,
    pub subscribe_denied: u64,
    pub messages_in: u64,
    pub messages_out: u64,
    pub events_live_dispatched: u64,
    pub events_replayed: u64,
    pub replay_gap_total: u64,
    pub replay_gap_alerts: u64,
    pub outbound_queue_dropped: u64,
    pub slow_consumer_disconnects: u64,
    pub client_op_auth: u64,
    pub client_op_subscribe: u64,
    pub client_op_unsubscribe: u64,
    pub client_op_ack: u64,
    pub client_op_ping: u64,
    pub invalid_messages: u64,
    pub errors_unauthorized: u64,
    pub errors_channel_disabled: u64,
    pub errors_forbidden: u64,
    pub errors_invalid_message: u64,
    pub errors_rate_limited: u64,
    pub replay_gap_slo: RealtimeReplayGapSloSnapshot,
    pub inbound_process_latency_ms: RealtimeHistogramSnapshot,
    pub pubsub_dispatch_latency_ms: RealtimeHistogramSnapshot,
    pub channels: Vec<RealtimeChannelMetricsSnapshot>,
    pub scopes: Vec<RealtimeScopeMetricsSnapshot>,
}

#[derive(Debug, Clone, Copy, Default)]
struct ChannelRuntimeLimits {
    max_message_bytes: Option<usize>,
    max_frame_bytes: Option<usize>,
    max_messages_per_sec: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
struct ConnectionLimits {
    max_message_bytes: usize,
    max_frame_bytes: usize,
    max_messages_per_sec: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct StreamId {
    millis: u64,
    seq: u64,
}

impl StreamId {
    fn parse(raw: &str) -> Option<Self> {
        let value = raw.trim();
        let (millis, seq) = value.split_once('-')?;
        Some(Self {
            millis: millis.parse().ok()?,
            seq: seq.parse().ok()?,
        })
    }
}

#[derive(Debug, Clone)]
struct ReplayGapInfo {
    requested_since_id: String,
    first_available_id: String,
    last_available_id: String,
}

#[derive(Default)]
struct DispatchOutcome {
    delivered: u64,
    queue_dropped: u64,
    slow_disconnects: Vec<Uuid>,
    closed_connections: Vec<Uuid>,
}

#[derive(Clone)]
pub struct WsServerState {
    settings: Arc<core_config::Settings>,
    policies: ChannelPolicyRegistry,
    presence: PresenceManager,
    subscriber: RealtimeSubscriber,
    redis_client: redis::Client,
    auth_resolver: AuthResolver,
    hub: Arc<RwLock<HubState>>,
    metrics: Arc<RealtimeMetrics>,
    channel_limits: Arc<HashMap<String, ChannelRuntimeLimits>>,
}

impl WsServerState {
    pub fn new(
        settings: Arc<core_config::Settings>,
        policies: ChannelPolicyRegistry,
        presence: PresenceManager,
        subscriber: RealtimeSubscriber,
        redis_url: &str,
        auth_resolver: AuthResolver,
    ) -> anyhow::Result<Self> {
        let channel_limits = settings
            .realtime
            .channels
            .iter()
            .map(|(name, cfg)| {
                (
                    normalize_channel(name),
                    ChannelRuntimeLimits {
                        max_message_bytes: cfg.max_message_bytes,
                        max_frame_bytes: cfg.max_frame_bytes,
                        max_messages_per_sec: cfg.max_messages_per_sec,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            settings,
            policies,
            presence,
            subscriber,
            redis_client: redis::Client::open(redis_url)?,
            auth_resolver,
            hub: Arc::new(RwLock::new(HubState::default())),
            metrics: Arc::new(RealtimeMetrics::default()),
            channel_limits: Arc::new(channel_limits),
        })
    }

    pub fn settings(&self) -> &Arc<core_config::Settings> {
        &self.settings
    }

    pub fn policies(&self) -> &ChannelPolicyRegistry {
        &self.policies
    }

    pub fn metrics_snapshot(&self) -> RealtimeMetricsSnapshot {
        let inbound_process_latency_ms = self
            .metrics
            .inbound_process_latency_ms
            .lock()
            .map(|value| value.snapshot())
            .unwrap_or_else(|_| RealtimeHistogram::with_buckets(&LATENCY_BUCKETS_MS).snapshot());
        let pubsub_dispatch_latency_ms = self
            .metrics
            .pubsub_dispatch_latency_ms
            .lock()
            .map(|value| value.snapshot())
            .unwrap_or_else(|_| RealtimeHistogram::with_buckets(&LATENCY_BUCKETS_MS).snapshot());
        RealtimeMetricsSnapshot {
            connections_opened: self.metrics.connections_opened.load(Ordering::Relaxed),
            connections_closed: self.metrics.connections_closed.load(Ordering::Relaxed),
            current_connections: self.metrics.current_connections.load(Ordering::Relaxed),
            auth_success: self.metrics.auth_success.load(Ordering::Relaxed),
            auth_failure: self.metrics.auth_failure.load(Ordering::Relaxed),
            subscribe_success: self.metrics.subscribe_success.load(Ordering::Relaxed),
            subscribe_denied: self.metrics.subscribe_denied.load(Ordering::Relaxed),
            messages_in: self.metrics.messages_in.load(Ordering::Relaxed),
            messages_out: self.metrics.messages_out.load(Ordering::Relaxed),
            events_live_dispatched: self.metrics.events_live_dispatched.load(Ordering::Relaxed),
            events_replayed: self.metrics.events_replayed.load(Ordering::Relaxed),
            replay_gap_total: self.metrics.replay_gap_total.load(Ordering::Relaxed),
            replay_gap_alerts: self.metrics.replay_gap_alerts.load(Ordering::Relaxed),
            outbound_queue_dropped: self.metrics.outbound_queue_dropped.load(Ordering::Relaxed),
            slow_consumer_disconnects: self
                .metrics
                .slow_consumer_disconnects
                .load(Ordering::Relaxed),
            client_op_auth: self.metrics.client_op_auth.load(Ordering::Relaxed),
            client_op_subscribe: self.metrics.client_op_subscribe.load(Ordering::Relaxed),
            client_op_unsubscribe: self.metrics.client_op_unsubscribe.load(Ordering::Relaxed),
            client_op_ack: self.metrics.client_op_ack.load(Ordering::Relaxed),
            client_op_ping: self.metrics.client_op_ping.load(Ordering::Relaxed),
            invalid_messages: self.metrics.invalid_messages.load(Ordering::Relaxed),
            errors_unauthorized: self.metrics.errors_unauthorized.load(Ordering::Relaxed),
            errors_channel_disabled: self.metrics.errors_channel_disabled.load(Ordering::Relaxed),
            errors_forbidden: self.metrics.errors_forbidden.load(Ordering::Relaxed),
            errors_invalid_message: self.metrics.errors_invalid_message.load(Ordering::Relaxed),
            errors_rate_limited: self.metrics.errors_rate_limited.load(Ordering::Relaxed),
            replay_gap_slo: self.metrics.replay_gap_slo_snapshot(
                self.settings.realtime.replay_gap_alert_threshold,
                self.settings.realtime.replay_gap_alert_window_secs,
            ),
            inbound_process_latency_ms,
            pubsub_dispatch_latency_ms,
            channels: self.metrics.channels_snapshot(),
            scopes: self.metrics.scopes_snapshot(),
        }
    }

    fn global_connection_limits(&self) -> ConnectionLimits {
        ConnectionLimits {
            max_message_bytes: self.settings.realtime.max_message_bytes.max(1),
            max_frame_bytes: self.settings.realtime.max_frame_bytes.max(1),
            max_messages_per_sec: self.settings.realtime.max_messages_per_sec.max(1),
        }
    }

    fn resolve_connection_limits(
        &self,
        subscribed_channels: &HashMap<String, u32>,
    ) -> ConnectionLimits {
        let mut limits = self.global_connection_limits();
        for channel in subscribed_channels.keys() {
            if let Some(cfg) = self.channel_limits.get(channel) {
                if let Some(v) = cfg.max_message_bytes {
                    limits.max_message_bytes = limits.max_message_bytes.min(v.max(1));
                }
                if let Some(v) = cfg.max_frame_bytes {
                    limits.max_frame_bytes = limits.max_frame_bytes.min(v.max(1));
                }
                if let Some(v) = cfg.max_messages_per_sec {
                    limits.max_messages_per_sec = limits.max_messages_per_sec.min(v.max(1));
                }
            }
        }
        limits
    }

    fn checkpoint_key(subject_id: &str, channel: &str, room: Option<&str>) -> String {
        let room_key = room.map(str::trim).filter(|r| !r.is_empty()).unwrap_or("_");
        format!("rt:checkpoint:{subject_id}:{channel}:{room_key}")
    }

    async fn load_checkpoint(
        &self,
        subject_id: &str,
        channel: &str,
        room: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        if !self.settings.realtime.checkpoint_enabled {
            return Ok(None);
        }
        let key = Self::checkpoint_key(subject_id, channel, room);
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let checkpoint: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
        Ok(checkpoint.filter(|v| !v.trim().is_empty()))
    }

    async fn save_checkpoint(
        &self,
        subject_id: &str,
        channel: &str,
        room: Option<&str>,
        delivery_id: &str,
    ) -> anyhow::Result<()> {
        if !self.settings.realtime.checkpoint_enabled {
            return Ok(());
        }
        let delivery_id = delivery_id.trim();
        if delivery_id.is_empty() {
            return Ok(());
        }
        let key = Self::checkpoint_key(subject_id, channel, room);
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: () = redis::cmd("SETEX")
            .arg(key)
            .arg(self.settings.realtime.checkpoint_ttl_secs.max(1))
            .arg(delivery_id)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    pub fn spawn_pubsub_loop(&self) {
        let state = self.clone();
        tokio::spawn(async move {
            loop {
                let callback_state = state.clone();
                let result = state
                    .subscriber
                    .listen(move |envelope| {
                        let callback_state = callback_state.clone();
                        async move {
                            callback_state.route_pubsub(envelope).await;
                        }
                    })
                    .await;
                if let Err(err) = result {
                    warn!("realtime pubsub listen failed: {err}");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        });
    }

    pub async fn connection_count(&self) -> usize {
        self.hub.read().await.senders.len()
    }

    async fn register_connection(
        &self,
        conn_id: Uuid,
    ) -> (
        mpsc::Receiver<ServerMessage>,
        watch::Receiver<ConnectionCloseSignal>,
    ) {
        let (tx, rx) = mpsc::channel(self.settings.realtime.send_queue_capacity.max(1));
        let (close_tx, close_rx) = watch::channel(ConnectionCloseSignal::None);
        self.hub.write().await.senders.insert(
            conn_id,
            ConnectionHandle {
                sender: tx,
                close_tx,
            },
        );
        self.metrics
            .connections_opened
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .current_connections
            .fetch_add(1, Ordering::Relaxed);
        (rx, close_rx)
    }

    async fn remove_connection(
        &self,
        conn_id: Uuid,
    ) -> Option<watch::Sender<ConnectionCloseSignal>> {
        let mut hub = self.hub.write().await;
        let removed = hub.senders.remove(&conn_id)?;
        hub.subscriptions.retain(|_, subscribers| {
            subscribers.remove(&conn_id);
            !subscribers.is_empty()
        });
        let close_tx = removed.close_tx;
        drop(hub);
        self.metrics
            .connections_closed
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .current_connections
            .fetch_sub(1, Ordering::Relaxed);
        Some(close_tx)
    }

    async fn unregister_connection(&self, conn_id: Uuid) {
        let _ = self.remove_connection(conn_id).await;
    }

    async fn disconnect_slow_consumer(&self, conn_id: Uuid) {
        if let Some(close_tx) = self.remove_connection(conn_id).await {
            self.metrics
                .slow_consumer_disconnects
                .fetch_add(1, Ordering::Relaxed);
            let _ = close_tx.send(ConnectionCloseSignal::SlowConsumer);
        }
    }

    async fn subscribe_connection(&self, conn_id: Uuid, channel: &str, room: Option<&str>) {
        let key = SubscriptionKey {
            channel: normalize_channel(channel),
            room: room.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
        };
        let mut hub = self.hub.write().await;
        hub.subscriptions
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(conn_id);
    }

    async fn unsubscribe_connection(&self, conn_id: Uuid, channel: &str, room: Option<&str>) {
        let key = SubscriptionKey {
            channel: normalize_channel(channel),
            room: room.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
        };
        let mut hub = self.hub.write().await;
        if let Some(subscribers) = hub.subscriptions.get_mut(&key) {
            subscribers.remove(&conn_id);
            if subscribers.is_empty() {
                hub.subscriptions.remove(&key);
            }
        }
    }

    async fn subscribers_for(&self, channel: &str, room: Option<&str>) -> Vec<Uuid> {
        let channel = normalize_channel(channel);
        let hub = self.hub.read().await;
        if let Some(room) = room {
            let key = SubscriptionKey {
                channel,
                room: Some(room.trim().to_string()),
            };
            return hub
                .subscriptions
                .get(&key)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default();
        }

        let mut all = HashSet::new();
        for (key, subscribers) in &hub.subscriptions {
            if key.channel == channel {
                all.extend(subscribers.iter().copied());
            }
        }
        all.into_iter().collect()
    }

    async fn dispatch_to_connections(
        &self,
        conn_ids: &[Uuid],
        message: &ServerMessage,
    ) -> DispatchOutcome {
        let senders = {
            let hub = self.hub.read().await;
            conn_ids
                .iter()
                .filter_map(|id| hub.senders.get(id).map(|entry| (*id, entry.sender.clone())))
                .collect::<Vec<_>>()
        };
        let mut outcome = DispatchOutcome::default();
        for (conn_id, sender) in senders {
            match sender.try_send(message.clone()) {
                Ok(()) => {
                    outcome.delivered = outcome.delivered.saturating_add(1);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    outcome.queue_dropped = outcome.queue_dropped.saturating_add(1);
                    outcome.slow_disconnects.push(conn_id);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    outcome.queue_dropped = outcome.queue_dropped.saturating_add(1);
                    outcome.closed_connections.push(conn_id);
                }
            }
        }
        if outcome.delivered > 0 {
            self.metrics
                .messages_out
                .fetch_add(outcome.delivered, Ordering::Relaxed);
        }
        if outcome.queue_dropped > 0 {
            self.metrics
                .outbound_queue_dropped
                .fetch_add(outcome.queue_dropped, Ordering::Relaxed);
        }
        for conn_id in outcome.closed_connections.iter().copied() {
            self.unregister_connection(conn_id).await;
        }
        for conn_id in outcome.slow_disconnects.iter().copied() {
            self.disconnect_slow_consumer(conn_id).await;
        }
        outcome
    }

    async fn route_pubsub(&self, envelope: PubSubEnvelope) {
        let started_at = Instant::now();
        if !self.policies.global_enabled() {
            return;
        }
        let channel = normalize_channel(&envelope.channel);
        let room = envelope
            .room
            .as_ref()
            .and_then(|v| normalize_room(v.clone()));
        let policy = self.policies.policy(&channel);
        if !policy.enabled {
            return;
        }
        let subscribers = self.subscribers_for(&channel, room.as_deref()).await;
        if subscribers.is_empty() {
            return;
        }
        let channel_name = channel.clone();
        let message = ServerMessage::Event {
            channel,
            event: envelope.event,
            room: room.clone(),
            payload: envelope.payload,
            sent_at_unix_ms: envelope.sent_at_unix_ms,
            delivery_id: envelope.delivery_id,
        };
        let outcome = self.dispatch_to_connections(&subscribers, &message).await;
        self.metrics
            .events_live_dispatched
            .fetch_add(outcome.delivered, Ordering::Relaxed);
        self.metrics
            .add_events_live_dispatched_channel(&channel_name, outcome.delivered);
        self.metrics.add_scope_events_live_dispatched(
            &channel_name,
            room.as_deref(),
            outcome.delivered,
            e2e_latency_ms(envelope.sent_at_unix_ms),
        );
        self.metrics
            .observe_pubsub_dispatch_latency(duration_ms(started_at.elapsed()));
    }

    async fn emit_presence(&self, channel: &str, room: &str) {
        let online = match self.presence.count(channel, room).await {
            Ok(count) => count,
            Err(err) => {
                warn!("presence count failed for channel={channel}, room={room}: {err}");
                return;
            }
        };
        let subscribers = self.subscribers_for(channel, Some(room)).await;
        if subscribers.is_empty() {
            return;
        }
        let msg = ServerMessage::Presence {
            channel: normalize_channel(channel),
            room: room.to_string(),
            online,
        };
        let channel_name = normalize_channel(channel);
        let outcome = self.dispatch_to_connections(&subscribers, &msg).await;
        self.metrics
            .add_messages_out_channel(&channel_name, outcome.delivered);
    }

    fn effective_replay_limit(&self, requested: Option<u32>) -> usize {
        let min_limit = 1usize;
        let max_limit = self.settings.realtime.replay_limit_max.max(min_limit);
        let default_limit = self.settings.realtime.replay_limit_default.max(min_limit);
        let requested = requested.map(|v| v as usize).unwrap_or(default_limit);
        requested.clamp(min_limit, max_limit)
    }

    async fn stream_bounds(&self, channel: &str) -> anyhow::Result<Option<(String, String)>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let stream_key = format!("rt:stream:{channel}");

        let first: Vec<(String, Vec<(String, String)>)> = redis::cmd("XRANGE")
            .arg(&stream_key)
            .arg("-")
            .arg("+")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut conn)
            .await?;
        if first.is_empty() {
            return Ok(None);
        }

        let last: Vec<(String, Vec<(String, String)>)> = redis::cmd("XREVRANGE")
            .arg(&stream_key)
            .arg("+")
            .arg("-")
            .arg("COUNT")
            .arg(1)
            .query_async(&mut conn)
            .await?;
        if last.is_empty() {
            return Ok(None);
        }

        Ok(Some((first[0].0.clone(), last[0].0.clone())))
    }

    async fn stream_id_exists(&self, channel: &str, stream_id: &str) -> anyhow::Result<bool> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let stream_key = format!("rt:stream:{channel}");
        let entries: Vec<(String, Vec<(String, String)>)> = redis::cmd("XRANGE")
            .arg(&stream_key)
            .arg(stream_id)
            .arg(stream_id)
            .arg("COUNT")
            .arg(1)
            .query_async(&mut conn)
            .await?;
        Ok(!entries.is_empty())
    }

    async fn detect_replay_gap(
        &self,
        channel: &str,
        since_id: &str,
    ) -> anyhow::Result<Option<ReplayGapInfo>> {
        if !matches!(
            self.settings.realtime.delivery_mode,
            RealtimeDeliveryMode::Durable
        ) {
            return Ok(None);
        }

        let Some(requested) = StreamId::parse(since_id) else {
            return Ok(None);
        };
        let Some((first_available_id, last_available_id)) = self.stream_bounds(channel).await?
        else {
            return Ok(None);
        };
        let Some(first_available) = StreamId::parse(&first_available_id) else {
            return Ok(None);
        };
        let Some(last_available) = StreamId::parse(&last_available_id) else {
            return Ok(None);
        };

        if requested < first_available {
            return Ok(Some(ReplayGapInfo {
                requested_since_id: since_id.to_string(),
                first_available_id,
                last_available_id,
            }));
        }
        if requested > last_available {
            return Ok(None);
        }
        if !self.stream_id_exists(channel, since_id).await? {
            return Ok(Some(ReplayGapInfo {
                requested_since_id: since_id.to_string(),
                first_available_id,
                last_available_id,
            }));
        }
        Ok(None)
    }

    async fn replay_to_connection(
        &self,
        conn_id: Uuid,
        channel: &str,
        room: Option<&str>,
        since_id: Option<&str>,
        replay_limit: Option<u32>,
    ) -> anyhow::Result<usize> {
        if !matches!(
            self.settings.realtime.delivery_mode,
            RealtimeDeliveryMode::Durable
        ) {
            return Ok(0);
        }
        if since_id.is_none() && replay_limit.is_none() {
            return Ok(0);
        }

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let limit = self.effective_replay_limit(replay_limit);
        let stream_key = format!("rt:stream:{channel}");

        let mut entries: Vec<(String, Vec<(String, String)>)> = if let Some(since_id) = since_id {
            let since_id = since_id.trim();
            if since_id.is_empty() {
                vec![]
            } else {
                let start = format!("({since_id}");
                redis::cmd("XRANGE")
                    .arg(&stream_key)
                    .arg(start)
                    .arg("+")
                    .arg("COUNT")
                    .arg(limit)
                    .query_async(&mut conn)
                    .await?
            }
        } else {
            redis::cmd("XREVRANGE")
                .arg(&stream_key)
                .arg("+")
                .arg("-")
                .arg("COUNT")
                .arg(limit)
                .query_async(&mut conn)
                .await?
        };

        if since_id.is_none() {
            entries.reverse();
        }

        let mut sent = 0usize;
        for (delivery_id, fields) in entries {
            let payload = fields
                .iter()
                .find_map(|(key, value)| (key == "data").then_some(value));
            let Some(payload) = payload else {
                continue;
            };
            let Ok(mut envelope) = serde_json::from_str::<PubSubEnvelope>(payload) else {
                continue;
            };
            envelope.delivery_id = Some(delivery_id);
            let normalized_channel = normalize_channel(&envelope.channel);
            if normalized_channel != channel {
                continue;
            }
            if let Some(expected_room) = room {
                if envelope.room.as_deref() != Some(expected_room) {
                    continue;
                }
            }
            let event_room = envelope.room.clone();
            if let Ok(true) = send_one(
                self,
                conn_id,
                ServerMessage::Event {
                    channel: normalized_channel,
                    event: envelope.event,
                    room: event_room.clone(),
                    payload: envelope.payload,
                    sent_at_unix_ms: envelope.sent_at_unix_ms,
                    delivery_id: envelope.delivery_id,
                },
            )
            .await
            {
                sent = sent.saturating_add(1);
                self.metrics.add_scope_events_replayed(
                    channel,
                    event_room.as_deref(),
                    1,
                    e2e_latency_ms(envelope.sent_at_unix_ms),
                );
            }
        }

        if sent > 0 {
            self.metrics
                .events_replayed
                .fetch_add(sent as u64, Ordering::Relaxed);
            self.metrics
                .add_events_replayed_channel(channel, sent as u64);
        }

        Ok(sent)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PresenceKey {
    channel: String,
    room: String,
}

#[derive(Debug)]
struct RateCounter {
    started_at: Instant,
    count: u32,
    limit: u32,
}

enum SocketReadEvent {
    ForcedClose(ConnectionCloseSignal),
    Frame(Option<Result<Message, axum::Error>>),
}

impl RateCounter {
    fn new(limit: u32) -> Self {
        Self {
            started_at: Instant::now(),
            count: 0,
            limit: limit.max(1),
        }
    }

    fn allow(&mut self) -> bool {
        if self.started_at.elapsed() >= Duration::from_secs(RATE_WINDOW_SECS) {
            self.started_at = Instant::now();
            self.count = 0;
        }
        self.count = self.count.saturating_add(1);
        self.count <= self.limit
    }

    fn set_limit(&mut self, limit: u32) {
        self.limit = limit.max(1);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<WsServerState>,
) -> Response {
    if !state.policies.global_enabled() {
        return ws_http_error(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::ChannelDisabled,
            "Realtime websocket is disabled",
        );
    }

    if state.connection_count().await >= state.settings.realtime.max_connections {
        return ws_http_error(
            StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::RateLimited,
            "Realtime connection limit reached",
        );
    }

    let global_limits = state.global_connection_limits();
    let policy_metadata = policy_metadata_from_headers(&headers);
    ws.max_message_size(global_limits.max_message_bytes)
        .max_frame_size(global_limits.max_frame_bytes)
        .on_upgrade(move |socket| handle_socket(state, socket, policy_metadata))
}

async fn handle_socket(state: WsServerState, socket: WebSocket, policy_metadata: PolicyMetadata) {
    let conn_id = Uuid::new_v4();
    let (mut outbound_rx, mut close_rx) = state.register_connection(conn_id).await;
    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut close_rx_writer = close_rx.clone();

    let writer = tokio::spawn(async move {
        let mut close_signal = ConnectionCloseSignal::None;
        loop {
            tokio::select! {
                changed = close_rx_writer.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    let signal = close_rx_writer.borrow().clone();
                    if !matches!(signal, ConnectionCloseSignal::None) {
                        close_signal = signal;
                        break;
                    }
                }
                next = outbound_rx.recv() => {
                    let Some(message) = next else {
                        break;
                    };
                    let Ok(body) = serde_json::to_string(&message) else {
                        continue;
                    };
                    if ws_tx.send(Message::Text(body.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
        if matches!(close_signal, ConnectionCloseSignal::SlowConsumer) {
            let _ = ws_tx
                .send(Message::Close(Some(CloseFrame {
                    code: close_code::POLICY,
                    reason: "slow consumer disconnected".into(),
                })))
                .await;
        } else {
            let _ = ws_tx.send(Message::Close(None)).await;
        }
    });

    let mut subscribed_channel_counts = HashMap::<String, u32>::new();
    let mut active_limits = state.global_connection_limits();
    let mut rate = RateCounter::new(active_limits.max_messages_per_sec);
    let mut auth = ConnectionAuthState::default();
    let mut presence_keys = HashSet::<PresenceKey>::new();

    let auth_timeout_secs = state.settings.realtime.heartbeat_secs.max(1) * AUTH_TIMEOUT_MULTIPLIER;
    let read_timeout_secs = state.settings.realtime.heartbeat_secs.max(1) * READ_TIMEOUT_MULTIPLIER;

    loop {
        let timeout_secs = if state.settings.realtime.require_auth && !auth.is_authenticated() {
            auth_timeout_secs
        } else {
            read_timeout_secs
        };
        let next = timeout(Duration::from_secs(timeout_secs), async {
            tokio::select! {
                changed = close_rx.changed() => {
                    if changed.is_ok() {
                        SocketReadEvent::ForcedClose(close_rx.borrow().clone())
                    } else {
                        SocketReadEvent::Frame(None)
                    }
                }
                frame = ws_rx.next() => SocketReadEvent::Frame(frame),
            }
        })
        .await;
        let frame = match next {
            Ok(SocketReadEvent::ForcedClose(ConnectionCloseSignal::None)) => continue,
            Ok(SocketReadEvent::ForcedClose(ConnectionCloseSignal::SlowConsumer)) => {
                let _ = send_error(
                    &state,
                    conn_id,
                    ErrorCode::RateLimited,
                    "Connection closed: slow consumer",
                )
                .await;
                break;
            }
            Ok(SocketReadEvent::Frame(Some(Ok(frame)))) => frame,
            Ok(SocketReadEvent::Frame(Some(Err(err)))) => {
                warn!("websocket read error: {err}");
                break;
            }
            Ok(SocketReadEvent::Frame(None)) => break,
            Err(_) => {
                if state.settings.realtime.require_auth && !auth.is_authenticated() {
                    let _ = send_error(
                        &state,
                        conn_id,
                        ErrorCode::Unauthorized,
                        "Authentication timeout",
                    )
                    .await;
                }
                break;
            }
        };

        state.metrics.messages_in.fetch_add(1, Ordering::Relaxed);

        if !rate.allow() {
            let _ = send_error(
                &state,
                conn_id,
                ErrorCode::RateLimited,
                "Rate limit exceeded",
            )
            .await;
            break;
        }

        match frame {
            Message::Text(text) => {
                let text = text.to_string();
                let text_limit = active_limits
                    .max_message_bytes
                    .min(active_limits.max_frame_bytes);
                if text.len() > text_limit {
                    let _ = send_error(
                        &state,
                        conn_id,
                        ErrorCode::InvalidMessage,
                        "Message exceeds maximum size",
                    )
                    .await;
                    break;
                }
                let parsed = serde_json::from_str::<ClientMessage>(&text);
                let msg = match parsed {
                    Ok(msg) => msg,
                    Err(_) => {
                        state.metrics.incr_invalid_message();
                        let _ = send_error(
                            &state,
                            conn_id,
                            ErrorCode::InvalidMessage,
                            "Invalid websocket message",
                        )
                        .await;
                        continue;
                    }
                };
                let started = Instant::now();
                if process_message(
                    &state,
                    conn_id,
                    msg,
                    &mut auth,
                    &mut presence_keys,
                    &mut subscribed_channel_counts,
                    &policy_metadata,
                )
                .await
                {
                    state
                        .metrics
                        .observe_inbound_process_latency(duration_ms(started.elapsed()));
                    break;
                }
                state
                    .metrics
                    .observe_inbound_process_latency(duration_ms(started.elapsed()));
                active_limits = state.resolve_connection_limits(&subscribed_channel_counts);
                rate.set_limit(active_limits.max_messages_per_sec);
            }
            Message::Binary(_) => {
                state.metrics.incr_invalid_message();
                let _ = send_error(
                    &state,
                    conn_id,
                    ErrorCode::InvalidMessage,
                    "Binary websocket messages are not supported",
                )
                .await;
            }
            Message::Ping(_) => {
                let _ = send_one(&state, conn_id, ServerMessage::Pong).await;
            }
            Message::Pong(_) => {}
            Message::Close(_) => break,
        }
    }

    if let Some(identity) = auth.identity.as_ref() {
        for key in presence_keys {
            if let Err(err) = state
                .presence
                .remove(&key.channel, &key.room, &identity.subject_id, conn_id)
                .await
            {
                warn!(
                    "presence remove failed for channel={}, room={}: {}",
                    key.channel, key.room, err
                );
                continue;
            }
            state.emit_presence(&key.channel, &key.room).await;
        }
    }

    state.unregister_connection(conn_id).await;
    writer.abort();
}

async fn process_message(
    state: &WsServerState,
    conn_id: Uuid,
    msg: ClientMessage,
    auth: &mut ConnectionAuthState,
    presence_keys: &mut HashSet<PresenceKey>,
    subscribed_channel_counts: &mut HashMap<String, u32>,
    policy_metadata: &PolicyMetadata,
) -> bool {
    match msg {
        ClientMessage::Auth { token } => {
            state.metrics.incr_op_auth();
            if token.trim().is_empty() {
                state.metrics.incr_invalid_message();
                state.metrics.auth_failure.fetch_add(1, Ordering::Relaxed);
                let _ =
                    send_error(state, conn_id, ErrorCode::Unauthorized, "Token is required").await;
                return false;
            }
            let identity = (state.auth_resolver)(token).await;
            let Some(identity) = identity else {
                state.metrics.auth_failure.fetch_add(1, Ordering::Relaxed);
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::Unauthorized,
                    "Invalid or expired token",
                )
                .await;
                return false;
            };
            auth.authenticated_at = Some(OffsetDateTime::now_utc());
            auth.identity = Some(identity.clone());
            state.metrics.auth_success.fetch_add(1, Ordering::Relaxed);
            let _ = send_one(
                state,
                conn_id,
                ServerMessage::AuthOk {
                    identity: AuthIdentityPublic {
                        subject_id: identity.subject_id,
                        guard: identity.guard,
                        abilities: identity.abilities,
                    },
                },
            )
            .await;
            false
        }
        ClientMessage::Subscribe {
            channel,
            room,
            since_id,
            replay_limit,
        } => {
            state.metrics.incr_op_subscribe();
            let channel = normalize_channel(&channel);
            if channel.is_empty() {
                state.metrics.incr_invalid_message();
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::InvalidMessage,
                    "Channel is required",
                )
                .await;
                return false;
            }
            let room = room.and_then(normalize_room);
            let client_since_id = since_id
                .and_then(|value| {
                    let value = value.trim().to_string();
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                })
                .filter(|value| !value.is_empty());
            if let Some(since_id) = client_since_id.as_deref() {
                if StreamId::parse(since_id).is_none() {
                    state.metrics.incr_invalid_message();
                    let _ = send_error(
                        state,
                        conn_id,
                        ErrorCode::InvalidMessage,
                        "since_id must use redis stream id format '<millis>-<seq>'",
                    )
                    .await;
                    return false;
                }
            }
            if let Err((code, message)) = validate_subscription(
                state,
                auth,
                conn_id,
                &channel,
                room.as_deref(),
                policy_metadata,
            ) {
                state
                    .metrics
                    .subscribe_denied
                    .fetch_add(1, Ordering::Relaxed);
                state.metrics.incr_subscribe_denied_channel(&channel);
                let _ = send_error(state, conn_id, code, &message).await;
                return false;
            }
            state
                .subscribe_connection(conn_id, &channel, room.as_deref())
                .await;
            state
                .metrics
                .subscribe_success
                .fetch_add(1, Ordering::Relaxed);
            state.metrics.incr_subscribe_success_channel(&channel);
            *subscribed_channel_counts
                .entry(channel.clone())
                .or_insert(0) += 1;

            let mut effective_since_id = client_since_id;
            if effective_since_id.is_none() && state.settings.realtime.checkpoint_enabled {
                if let Some(identity) = auth.identity.as_ref() {
                    if let Ok(saved) = state
                        .load_checkpoint(&identity.subject_id, &channel, room.as_deref())
                        .await
                    {
                        if saved.as_deref().and_then(StreamId::parse).is_some() {
                            effective_since_id = saved;
                        }
                    }
                }
            }

            if let Some(since_id) = effective_since_id.clone() {
                match state.detect_replay_gap(&channel, &since_id).await {
                    Ok(Some(gap)) => {
                        state.metrics.record_replay_gap(
                            &channel,
                            room.as_deref(),
                            state.settings.realtime.replay_gap_alert_threshold,
                            state.settings.realtime.replay_gap_alert_window_secs,
                        );
                        let _ = send_one(
                            state,
                            conn_id,
                            ServerMessage::ReplayGap {
                                channel: channel.clone(),
                                room: room.clone(),
                                requested_since_id: gap.requested_since_id,
                                first_available_id: gap.first_available_id,
                                last_available_id: gap.last_available_id,
                            },
                        )
                        .await;
                        // Replay from earliest available stream entries when client's cursor is stale.
                        effective_since_id = Some("0-0".to_string());
                    }
                    Ok(None) => {}
                    Err(err) => {
                        warn!(
                            "failed to detect replay gap for channel={}, room={}: {}",
                            channel,
                            room.as_deref().unwrap_or(""),
                            err
                        );
                    }
                }
            }

            if let Ok(replayed) = state
                .replay_to_connection(
                    conn_id,
                    &channel,
                    room.as_deref(),
                    effective_since_id.as_deref(),
                    replay_limit,
                )
                .await
            {
                if replayed > 0 {
                    tracing::debug!(
                        channel = %channel,
                        room = room.as_deref().unwrap_or(""),
                        replayed,
                        "replayed durable realtime events"
                    );
                }
            }

            let policy = state.policies.policy(&channel);
            if policy.presence_enabled {
                if let (Some(room), Some(identity)) = (room.as_deref(), auth.identity.as_ref()) {
                    if let Err(err) = state
                        .presence
                        .touch(&channel, room, &identity.subject_id, conn_id)
                        .await
                    {
                        warn!(
                            "presence touch failed for channel={}, room={}: {}",
                            channel, room, err
                        );
                    } else {
                        presence_keys.insert(PresenceKey {
                            channel: channel.clone(),
                            room: room.to_string(),
                        });
                        state.emit_presence(&channel, room).await;
                    }
                }
            }
            false
        }
        ClientMessage::Unsubscribe { channel, room } => {
            state.metrics.incr_op_unsubscribe();
            let channel = normalize_channel(&channel);
            let room = room.and_then(normalize_room);
            state
                .unsubscribe_connection(conn_id, &channel, room.as_deref())
                .await;
            if let Some(count) = subscribed_channel_counts.get_mut(&channel) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    subscribed_channel_counts.remove(&channel);
                }
            }

            let policy = state.policies.policy(&channel);
            if policy.presence_enabled {
                if let (Some(room), Some(identity)) = (room.as_deref(), auth.identity.as_ref()) {
                    let _ = state
                        .presence
                        .remove(&channel, room, &identity.subject_id, conn_id)
                        .await;
                    presence_keys.remove(&PresenceKey {
                        channel: channel.clone(),
                        room: room.to_string(),
                    });
                    state.emit_presence(&channel, room).await;
                }
            }
            false
        }
        ClientMessage::Ack {
            channel,
            room,
            delivery_id,
        } => {
            state.metrics.incr_op_ack();
            let channel = normalize_channel(&channel);
            let room = room.and_then(normalize_room);
            if channel.is_empty() {
                state.metrics.incr_invalid_message();
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::InvalidMessage,
                    "Channel is required",
                )
                .await;
                return false;
            }
            if delivery_id.trim().is_empty() {
                state.metrics.incr_invalid_message();
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::InvalidMessage,
                    "delivery_id is required",
                )
                .await;
                return false;
            }
            if StreamId::parse(&delivery_id).is_none() {
                state.metrics.incr_invalid_message();
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::InvalidMessage,
                    "delivery_id must use redis stream id format '<millis>-<seq>'",
                )
                .await;
                return false;
            }
            if state.settings.realtime.require_auth && auth.identity.is_none() {
                let _ = send_error(
                    state,
                    conn_id,
                    ErrorCode::Unauthorized,
                    "Authentication required",
                )
                .await;
                return false;
            }
            if let Some(identity) = auth.identity.as_ref() {
                let _ = state
                    .save_checkpoint(
                        &identity.subject_id,
                        &channel,
                        room.as_deref(),
                        &delivery_id,
                    )
                    .await;
            }
            let _ = send_one(
                state,
                conn_id,
                ServerMessage::AckOk {
                    channel,
                    room,
                    delivery_id,
                },
            )
            .await;
            false
        }
        ClientMessage::Ping => {
            state.metrics.incr_op_ping();
            if let Some(identity) = auth.identity.as_ref() {
                let keys = presence_keys.clone();
                for key in keys {
                    let _ = state
                        .presence
                        .touch(&key.channel, &key.room, &identity.subject_id, conn_id)
                        .await;
                }
            }
            let _ = send_one(state, conn_id, ServerMessage::Pong).await;
            false
        }
    }
}

fn validate_subscription(
    state: &WsServerState,
    auth: &ConnectionAuthState,
    conn_id: Uuid,
    channel: &str,
    room: Option<&str>,
    policy_metadata: &PolicyMetadata,
) -> Result<(), (ErrorCode, String)> {
    if !state.policies.global_enabled() {
        return Err((
            ErrorCode::ChannelDisabled,
            "Realtime websocket is disabled".to_string(),
        ));
    }

    let policy = state.policies.policy(channel);
    if !policy.enabled {
        return Err((
            ErrorCode::ChannelDisabled,
            format!("Channel '{channel}' is disabled"),
        ));
    }

    if state.settings.realtime.require_auth && auth.identity.is_none() {
        return Err((
            ErrorCode::Unauthorized,
            "Authentication required".to_string(),
        ));
    }

    if let Some(expected_guard) = policy.guard {
        let Some(identity) = auth.identity.as_ref() else {
            return Err((
                ErrorCode::Unauthorized,
                "Authentication required".to_string(),
            ));
        };
        if !identity.guard.eq_ignore_ascii_case(&expected_guard) {
            return Err((
                ErrorCode::Forbidden,
                format!(
                    "Guard '{}' cannot subscribe to channel '{channel}'",
                    identity.guard
                ),
            ));
        }
    }

    let context = PolicyContext::new(
        PolicyAction::Subscribe,
        PolicySource::WebSocket,
        channel,
        room.map(|v| v.to_string()),
    )
    .with_identity(auth.identity.as_ref())
    .with_connection_id(conn_id)
    .with_metadata(policy_metadata.clone());
    let decision = state.policies.authorize_subscribe(&context);
    if !decision.allowed {
        return Err(decision.into_error(format!("Subscription denied for channel '{channel}'")));
    }

    Ok(())
}

fn normalize_channel(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn normalize_room(raw: String) -> Option<String> {
    let room = raw.trim().to_string();
    if room.is_empty() {
        None
    } else {
        Some(room)
    }
}

fn policy_metadata_from_headers(headers: &HeaderMap) -> PolicyMetadata {
    let mut metadata = PolicyMetadata::default();
    metadata.request_id = header_value(headers, "x-request-id");
    metadata.user_agent = header_value(headers, "user-agent");
    metadata.tenant_id = header_value(headers, "x-tenant-id");
    metadata.remote_addr = header_value(headers, "x-forwarded-for").map(|raw| {
        raw.split(',')
            .next()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(raw.as_str())
            .to_string()
    });
    metadata
}

fn header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}

fn e2e_latency_ms(sent_at_unix_ms: i64) -> Option<u64> {
    let now = now_unix_ms();
    if now <= sent_at_unix_ms {
        return Some(0);
    }
    Some((now - sent_at_unix_ms) as u64)
}

fn now_unix_ms() -> i64 {
    (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}

fn ws_http_error(status: StatusCode, code: ErrorCode, message: &str) -> Response {
    (
        status,
        Json(json!({
            "ok": false,
            "error": {
                "code": code,
                "message": message
            }
        })),
    )
        .into_response()
}

async fn send_error(
    state: &WsServerState,
    conn_id: Uuid,
    code: ErrorCode,
    message: &str,
) -> anyhow::Result<()> {
    state.metrics.observe_error_code(code.clone());
    let _ = send_one(
        state,
        conn_id,
        ServerMessage::Error {
            code,
            message: message.to_string(),
        },
    )
    .await?;
    Ok(())
}

async fn send_one(
    state: &WsServerState,
    conn_id: Uuid,
    message: ServerMessage,
) -> anyhow::Result<bool> {
    let sender = {
        let hub = state.hub.read().await;
        hub.senders.get(&conn_id).map(|entry| entry.sender.clone())
    };
    if let Some(sender) = sender {
        match sender.try_send(message) {
            Ok(()) => {
                state.metrics.messages_out.fetch_add(1, Ordering::Relaxed);
                return Ok(true);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                state
                    .metrics
                    .outbound_queue_dropped
                    .fetch_add(1, Ordering::Relaxed);
                state.disconnect_slow_consumer(conn_id).await;
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                state
                    .metrics
                    .outbound_queue_dropped
                    .fetch_add(1, Ordering::Relaxed);
                state.unregister_connection(conn_id).await;
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{header_value, policy_metadata_from_headers, send_one, StreamId, WsServerState};
    use crate::registry::{AllowAllSubscribeAuthorizer, ChannelPolicy, ChannelPolicyRegistry};
    use crate::types::ServerMessage;
    use crate::AuthResolver;
    use axum::http::{HeaderMap, HeaderValue};
    use core_config::{
        AppSettings, AuthSettings, CdnSettings, DataTableUnknownFilterMode, DbSettings,
        GuardConfig, HttpLogSettings, MailSettings, MiddlewareSettings, RealtimeChannelConfig,
        RealtimeDeliveryMode, RealtimeSettings, RedisSettings, S3Settings, ServerSettings,
        Settings, WorkerSettings,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    fn test_settings(redis_url: &str, send_queue_capacity: usize) -> Arc<Settings> {
        let mut channels = HashMap::new();
        channels.insert(
            "unit".to_string(),
            RealtimeChannelConfig {
                enabled: true,
                guard: None,
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
        Arc::new(Settings {
            app: AppSettings {
                name: "realtime-test".to_string(),
                env: "test".to_string(),
                key: "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
                enable_docs: false,
                docs_path: "/framework-documentation".to_string(),
                enable_openapi_docs: false,
                openapi_docs_path: "/openapi".to_string(),
                openapi_json_path: "/openapi.json".to_string(),
                default_per_page: 30,
                datatable_unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
            },
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
                max_connections: 1000,
                max_message_bytes: 64 * 1024,
                max_frame_bytes: 64 * 1024,
                max_messages_per_sec: 200,
                send_queue_capacity: send_queue_capacity.max(1),
                require_auth: true,
                checkpoint_enabled: false,
                checkpoint_ttl_secs: 3600,
                delivery_mode: RealtimeDeliveryMode::AtMostOnce,
                stream_max_len: 1024,
                stream_retention_secs: 0,
                replay_limit_default: 200,
                replay_limit_max: 1000,
                replay_gap_alert_threshold: 100,
                replay_gap_alert_window_secs: 60,
                channels,
            },
            db: DbSettings {
                url: "postgres://localhost/test".to_string(),
                max_connections: 1,
                connect_timeout: Duration::from_secs(1),
            },
            redis: RedisSettings {
                url: redis_url.to_string(),
                prefix: None,
            },
            s3: S3Settings {
                endpoint: String::new(),
                region: "auto".to_string(),
                bucket: String::new(),
                access_key: String::new(),
                secret_key: String::new(),
                force_path_style: false,
            },
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
                    .expect("timezone"),
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
            mail: MailSettings {
                enable: false,
                driver: "log".to_string(),
                host: String::new(),
                port: 0,
                username: None,
                password: None,
                from_address: "test@example.com".to_string(),
            },
            http_log: HttpLogSettings {
                webhook_enabled: false,
                webhook_paths: vec![],
                client_enabled: false,
                retention_days: 7,
            },
        })
    }

    async fn test_state(send_queue_capacity: usize) -> anyhow::Result<WsServerState> {
        let redis_url = "redis://127.0.0.1:6379/0";
        let mut policies = HashMap::new();
        policies.insert(
            "unit".to_string(),
            ChannelPolicy {
                enabled: true,
                guard: None,
                presence_enabled: true,
            },
        );
        let registry =
            ChannelPolicyRegistry::new(true, policies, Arc::new(AllowAllSubscribeAuthorizer));
        let presence = crate::presence::PresenceManager::new(redis_url, 30)?;
        let subscriber = crate::pubsub::RealtimeSubscriber::new(redis_url)?;
        let auth_resolver: AuthResolver = Arc::new(|_token: String| Box::pin(async { None }));
        WsServerState::new(
            test_settings(redis_url, send_queue_capacity),
            registry,
            presence,
            subscriber,
            redis_url,
            auth_resolver,
        )
    }

    #[test]
    fn stream_id_parse_accepts_valid_ids() {
        let parsed = StreamId::parse("1739223597000-4");
        assert_eq!(
            parsed,
            Some(StreamId {
                millis: 1_739_223_597_000,
                seq: 4
            })
        );
    }

    #[test]
    fn stream_id_parse_rejects_invalid_ids() {
        assert!(StreamId::parse("").is_none());
        assert!(StreamId::parse("abc").is_none());
        assert!(StreamId::parse("1").is_none());
        assert!(StreamId::parse("1-a").is_none());
    }

    #[tokio::test]
    async fn full_outbound_queue_disconnects_slow_consumer() {
        let state = test_state(1).await.expect("state");
        let conn_id = Uuid::new_v4();
        let (_rx, _close_rx) = state.register_connection(conn_id).await;

        let first = send_one(&state, conn_id, ServerMessage::Pong)
            .await
            .expect("send first");
        let second = send_one(&state, conn_id, ServerMessage::Pong)
            .await
            .expect("send second");

        assert!(first);
        assert!(!second);
        assert_eq!(state.connection_count().await, 0);

        let metrics = state.metrics_snapshot();
        assert_eq!(metrics.outbound_queue_dropped, 1);
        assert_eq!(metrics.slow_consumer_disconnects, 1);
    }

    #[test]
    fn policy_metadata_extracts_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req-123"));
        headers.insert("x-tenant-id", HeaderValue::from_static("42"));
        headers.insert("user-agent", HeaderValue::from_static("test-agent"));
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.9, 10.0.0.1"),
        );

        let metadata = policy_metadata_from_headers(&headers);
        assert_eq!(metadata.request_id.as_deref(), Some("req-123"));
        assert_eq!(metadata.tenant_id.as_deref(), Some("42"));
        assert_eq!(metadata.user_agent.as_deref(), Some("test-agent"));
        assert_eq!(metadata.remote_addr.as_deref(), Some("203.0.113.9"));
        assert_eq!(
            header_value(&headers, "x-request-id").as_deref(),
            Some("req-123")
        );
    }
}
