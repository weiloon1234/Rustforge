use crate::types::{PubSubEnvelope, RealtimeEvent, RealtimeTarget};
use anyhow::Result;
use core_config::RealtimeDeliveryMode;
use redis::AsyncCommands;
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy)]
pub struct RealtimePublishSettings {
    pub delivery_mode: RealtimeDeliveryMode,
    pub stream_max_len: usize,
    pub stream_retention_secs: u64,
}

#[derive(Clone)]
pub struct RealtimePublisher {
    client: redis::Client,
    settings: RealtimePublishSettings,
}

impl RealtimePublisher {
    pub fn new(redis_url: &str) -> Result<Self> {
        Self::new_with_settings(
            redis_url,
            RealtimePublishSettings {
                delivery_mode: RealtimeDeliveryMode::AtMostOnce,
                stream_max_len: 100000,
                stream_retention_secs: 0,
            },
        )
    }

    pub fn from_realtime_settings(
        redis_url: &str,
        settings: &core_config::RealtimeSettings,
    ) -> Result<Self> {
        Self::new_with_settings(
            redis_url,
            RealtimePublishSettings {
                delivery_mode: settings.delivery_mode,
                stream_max_len: settings.stream_max_len,
                stream_retention_secs: settings.stream_retention_secs,
            },
        )
    }

    pub fn new_with_settings(redis_url: &str, settings: RealtimePublishSettings) -> Result<Self> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            settings,
        })
    }

    pub async fn publish<E: RealtimeEvent>(
        &self,
        target: RealtimeTarget,
        payload: &E,
    ) -> Result<()> {
        self.publish_raw(
            E::CHANNEL,
            E::EVENT,
            target.room.as_deref(),
            serde_json::to_value(payload)?,
        )
        .await
    }

    pub async fn publish_raw(
        &self,
        channel: &str,
        event: &str,
        room: Option<&str>,
        payload: Value,
    ) -> Result<()> {
        let now = OffsetDateTime::now_utc();
        let sent_at_unix_ms = now.unix_timestamp_nanos() / 1_000_000;
        let mut envelope = PubSubEnvelope {
            channel: channel.to_string(),
            event: event.to_string(),
            room: room.map(ToString::to_string),
            payload,
            sent_at_unix_ms: sent_at_unix_ms as i64,
            delivery_id: None,
        };

        let mut conn = self.client.get_multiplexed_async_connection().await?;

        if matches!(self.settings.delivery_mode, RealtimeDeliveryMode::Durable) {
            let stream_key = format!("rt:stream:{channel}");
            let stream_payload = serde_json::to_string(&envelope)?;
            let stream_id: String = redis::cmd("XADD")
                .arg(&stream_key)
                .arg("MAXLEN")
                .arg("~")
                .arg(self.settings.stream_max_len.max(1))
                .arg("*")
                .arg("data")
                .arg(stream_payload)
                .query_async(&mut conn)
                .await?;
            envelope.delivery_id = Some(stream_id);

            if self.settings.stream_retention_secs > 0 {
                let now_ms = now.unix_timestamp_nanos() / 1_000_000;
                let min_ms =
                    now_ms.saturating_sub((self.settings.stream_retention_secs as i128) * 1_000);
                let min_id = format!("{min_ms}-0");
                let _: usize = redis::cmd("XTRIM")
                    .arg(&stream_key)
                    .arg("MINID")
                    .arg("~")
                    .arg(min_id)
                    .query_async(&mut conn)
                    .await?;
            }
        }

        let topic = format!("rt:channel:{channel}");
        let body = serde_json::to_string(&envelope)?;
        let _: () = conn.publish(topic, body).await?;
        Ok(())
    }
}
