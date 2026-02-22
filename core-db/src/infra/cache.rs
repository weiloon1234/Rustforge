use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;

use core_config::RedisSettings;

#[derive(Clone)]
pub struct Cache {
    conn: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    prefix: Option<String>,
}

pub async fn create_cache(settings: &RedisSettings) -> Result<Cache> {
    let client = Client::open(settings.url.as_str())?;
    let conn = client.get_multiplexed_async_connection().await?;

    Ok(Cache {
        conn: Arc::new(Mutex::new(conn)),
        prefix: settings.prefix.clone(),
    })
}

impl Cache {
    fn key(&self, k: &str) -> String {
        match &self.prefix {
            Some(p) => format!("{}:{}", p, k),
            None => k.to_string(),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn.lock().await;
        Ok(conn.get(self.key(key)).await?)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.conn.lock().await;
        conn.set::<_, _, ()>(self.key(key), value).await?;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.lock().await;
        conn.del::<_, ()>(self.key(key)).await?;
        Ok(())
    }

    // ── TTL + Convenience ──────────────────────────────────────────

    pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()> {
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(self.key(key), value, ttl_secs).await?;
        Ok(())
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let mut conn = self.conn.lock().await;
        let val: i64 = conn.ttl(self.key(key)).await?;
        Ok(if val < 0 { None } else { Some(val) })
    }

    pub async fn forget(&self, key: &str) -> Result<()> {
        self.del(key).await
    }

    pub async fn has(&self, key: &str) -> Result<bool> {
        let mut conn = self.conn.lock().await;
        Ok(conn.exists(self.key(key)).await?)
    }

    // ── Typed JSON ─────────────────────────────────────────────────

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
            None => Ok(None),
        }
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        self.set(key, &raw).await
    }

    pub async fn set_json_ex<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        self.set_ex(key, &raw, ttl_secs).await
    }

    // ── Remember Pattern ───────────────────────────────────────────

    pub async fn remember<T, F, Fut>(&self, key: &str, ttl_secs: u64, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }
        let value = f().await?;
        self.set_json_ex(key, &value, ttl_secs).await?;
        Ok(value)
    }

    pub async fn remember_forever<T, F, Fut>(&self, key: &str, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }
        let value = f().await?;
        self.set_json(key, &value).await?;
        Ok(value)
    }

    // ── Atomic Counters ────────────────────────────────────────────

    pub async fn increment(&self, key: &str, by: i64) -> Result<i64> {
        let mut conn = self.conn.lock().await;
        Ok(conn.incr(self.key(key), by).await?)
    }

    pub async fn decrement(&self, key: &str, by: i64) -> Result<i64> {
        let mut conn = self.conn.lock().await;
        Ok(conn.decr(self.key(key), by).await?)
    }

    // ── Bulk Operations ────────────────────────────────────────────

    pub async fn many(&self, keys: &[&str]) -> Result<Vec<Option<String>>> {
        let mut conn = self.conn.lock().await;
        let prefixed: Vec<String> = keys.iter().map(|k| self.key(k)).collect();
        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&prefixed)
            .query_async(&mut *conn)
            .await?;
        Ok(results)
    }

    pub async fn put_many(&self, pairs: &[(&str, &str)]) -> Result<()> {
        let mut conn = self.conn.lock().await;
        let mut pipe = redis::pipe();
        for (k, v) in pairs {
            pipe.set(self.key(k), *v);
        }
        pipe.query_async::<()>(&mut *conn).await?;
        Ok(())
    }

    pub async fn flush_prefix(&self, prefix: &str) -> Result<()> {
        let mut conn = self.conn.lock().await;
        let pattern = format!("{}*", self.key(prefix));
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await?;
        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async::<()>(&mut *conn)
                .await?;
        }
        Ok(())
    }
}
