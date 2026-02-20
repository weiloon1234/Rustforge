#[derive(Debug, Clone)]
pub struct RealtimeIdempotency {
    redis_client: redis::Client,
    ttl_secs: u64,
    key_prefix: String,
}

impl RealtimeIdempotency {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        Self::with_options(redis_url, "rt:idem", 60 * 60 * 24)
    }

    pub fn with_options(
        redis_url: &str,
        key_prefix: impl Into<String>,
        ttl_secs: u64,
    ) -> anyhow::Result<Self> {
        let key_prefix = key_prefix.into();
        let key_prefix = key_prefix.trim();
        if key_prefix.is_empty() {
            anyhow::bail!("Realtime idempotency key_prefix cannot be empty");
        }
        Ok(Self {
            redis_client: redis::Client::open(redis_url)?,
            ttl_secs: ttl_secs.max(1),
            key_prefix: key_prefix.to_string(),
        })
    }

    /// Claim a delivery key once within TTL.
    ///
    /// Returns:
    /// - `true`  => first claim, caller should process
    /// - `false` => duplicate claim, caller should skip
    pub async fn claim_once(&self, scope: &str, delivery_id: &str) -> anyhow::Result<bool> {
        let scope = scope.trim();
        if scope.is_empty() {
            anyhow::bail!("Realtime idempotency scope cannot be empty");
        }
        let delivery_id = delivery_id.trim();
        if delivery_id.is_empty() {
            anyhow::bail!("Realtime idempotency delivery_id cannot be empty");
        }

        let key = format!("{}:{}:{}", self.key_prefix, scope, delivery_id);
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let set: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg("1")
            .arg("EX")
            .arg(self.ttl_secs)
            .arg("NX")
            .query_async(&mut conn)
            .await?;
        Ok(set.as_deref() == Some("OK"))
    }
}
