#![allow(dead_code)]
use std::sync::Arc;

use anyhow::Result;
use redis::{AsyncCommands, Client};
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
}
