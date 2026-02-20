#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder as S3ConfigBuilder, Region},
    primitives::ByteStream,
    Client,
};

use core_config::S3Settings;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> anyhow::Result<()>;
    async fn get(&self, key: &str) -> anyhow::Result<Bytes>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> anyhow::Result<Bytes> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(resp.body.collect().await?.into_bytes())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

pub async fn create_storage(settings: &S3Settings) -> anyhow::Result<Arc<dyn Storage>> {
    // R2 requires explicit endpoint + credentials
    if settings.endpoint.is_empty() {
        anyhow::bail!("S3 endpoint is required for R2");
    }

    let creds = Credentials::new(
        &settings.access_key,
        &settings.secret_key,
        None,
        None,
        "static",
    );

    let mut cfg = S3ConfigBuilder::new()
        .credentials_provider(creds)
        .region(Region::new(settings.region.clone()))
        .endpoint_url(settings.endpoint.clone());

    if settings.force_path_style {
        cfg = cfg.force_path_style(true);
    }

    let client = Client::from_conf(cfg.build());

    Ok(Arc::new(S3Storage::new(client, settings.bucket.clone())))
}
