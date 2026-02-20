use anyhow::Result;
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Clone)]
pub struct PresenceManager {
    client: redis::Client,
    ttl_secs: u64,
}

impl PresenceManager {
    pub fn new(redis_url: &str, ttl_secs: u64) -> Result<Self> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            ttl_secs: ttl_secs.max(1),
        })
    }

    fn key(channel: &str, room: &str, subject_id: &str, conn_id: Uuid) -> String {
        format!("rt:presence:{channel}:{room}:{subject_id}:{conn_id}")
    }

    fn scan_pattern(channel: &str, room: &str) -> String {
        format!("rt:presence:{channel}:{room}:*")
    }

    pub async fn touch(
        &self,
        channel: &str,
        room: &str,
        subject_id: &str,
        conn_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::key(channel, room, subject_id, conn_id);
        let _: () = conn.set_ex(key, "1", self.ttl_secs).await?;
        Ok(())
    }

    pub async fn remove(
        &self,
        channel: &str,
        room: &str,
        subject_id: &str,
        conn_id: Uuid,
    ) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::key(channel, room, subject_id, conn_id);
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn count(&self, channel: &str, room: &str) -> Result<u64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let pattern = Self::scan_pattern(channel, room);
        let keys: Vec<String> = conn.keys(pattern).await?;
        Ok(u64::try_from(keys.len()).unwrap_or(u64::MAX))
    }
}
