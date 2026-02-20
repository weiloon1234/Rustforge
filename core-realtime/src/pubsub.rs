use crate::types::PubSubEnvelope;
use anyhow::Result;
use futures_util::StreamExt;

#[derive(Clone)]
pub struct RealtimeSubscriber {
    client: redis::Client,
}

impl RealtimeSubscriber {
    pub fn new(redis_url: &str) -> Result<Self> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn listen<F, Fut>(&self, mut on_event: F) -> Result<()>
    where
        F: FnMut(PubSubEnvelope) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.psubscribe("rt:channel:*").await?;
        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let Ok(payload) = msg.get_payload::<String>() else {
                continue;
            };
            let Ok(event) = serde_json::from_str::<PubSubEnvelope>(&payload) else {
                continue;
            };
            on_event(event).await;
        }
        Ok(())
    }
}
