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
use sqlx::PgPool;

/// Cleanup old logs based on retention days
pub async fn cleanup_logs(db: &PgPool, retention_days: u64) -> Result<()> {
    if retention_days == 0 {
        return Ok(());
    }

    let interval = format!("{} days", retention_days);

    // Clean webhook logs (if table exists - assuming migrations are run)
    // using query (not query!) to avoid compile-time DB checks for library
    sqlx::query("DELETE FROM webhook_logs WHERE created_at < NOW() - $1::INTERVAL")
        .bind(&interval)
        .execute(db)
        .await?;

    // Clean client logs
    sqlx::query("DELETE FROM http_client_logs WHERE created_at < NOW() - $1::INTERVAL")
        .bind(&interval)
        .execute(db)
        .await?;

    Ok(())
}
