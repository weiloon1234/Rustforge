//! HTTP Traffic Logging
//!
//! This crate provides logging for both inbound webhook requests and outbound HTTP client requests.
//!
//! # Features
//! - `WebhookLogLayer` - Middleware for logging incoming webhook requests
//! - `HttpClient` - Wrapped reqwest client with request/response logging
//!
//! # Configuration
//! Controlled via environment variables:
//! - `HTTP_LOG_WEBHOOK_ENABLED` - Enable/disable webhook logging
//! - `HTTP_LOG_WEBHOOK_PATHS` - Comma-separated path prefixes to log (e.g., "/wh/,/webhook/")
//! - `HTTP_LOG_CLIENT_ENABLED` - Enable/disable HTTP client logging

pub mod http_client;
pub mod jobs;
pub mod models;
pub mod webhook_middleware;

pub use http_client::HttpClient;
pub use models::{HttpClientLog, WebhookLog};
pub use webhook_middleware::WebhookLogLayer;

use anyhow::Result;
use core_db::{
    common::sql::{DbConn, Op},
    generated::models::{HttpClientLog as HttpClientLogModel, WebhookLog as WebhookLogModel},
};
use sqlx::PgPool;
use time::OffsetDateTime;

/// Cleanup old logs based on retention days
pub async fn cleanup_logs(db: &PgPool, retention_days: u64) -> Result<()> {
    if retention_days == 0 {
        return Ok(());
    }

    let cutoff = OffsetDateTime::now_utc() - time::Duration::days(retention_days as i64);

    WebhookLogModel::new(DbConn::pool(db), None)
        .query()
        .where_created_at(Op::Lt, cutoff)
        .delete()
        .await?;

    HttpClientLogModel::new(DbConn::pool(db), None)
        .query()
        .where_created_at(Op::Lt, cutoff)
        .delete()
        .await?;

    Ok(())
}
