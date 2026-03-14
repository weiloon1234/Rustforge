//! Database access wrappers for HTTP traffic logs.

use core_db::{
    common::sql::DbConn,
    generated::models::{
        HttpClientLogCol, HttpClientLogModel, WebhookLogCol, WebhookLogModel,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

/// Webhook (inbound) request log API.
pub struct WebhookLog;

impl WebhookLog {
    /// Insert a new webhook log entry.
    pub async fn insert(
        pool: &PgPool,
        request_url: &str,
        request_method: &str,
        request_headers: Option<serde_json::Value>,
        request_body: Option<&str>,
        response_status: Option<i32>,
        response_body: Option<&str>,
        duration_ms: Option<i32>,
    ) -> anyhow::Result<Uuid> {
        let row = WebhookLogModel::create(DbConn::pool(pool))
            .set(WebhookLogCol::REQUEST_URL, request_url.to_string())?
            .set(WebhookLogCol::REQUEST_METHOD, request_method.to_string())?
            .set(WebhookLogCol::REQUEST_HEADERS, request_headers)?
            .set(WebhookLogCol::REQUEST_BODY, request_body.map(str::to_string))?
            .set(WebhookLogCol::RESPONSE_STATUS, response_status)?
            .set(WebhookLogCol::RESPONSE_BODY, response_body.map(str::to_string))?
            .set(WebhookLogCol::DURATION_MS, duration_ms)?
            .save()
            .await?;

        Ok(row.id)
    }
}

/// HTTP client (outbound) request log API.
pub struct HttpClientLog;

impl HttpClientLog {
    /// Insert a new HTTP client log entry.
    pub async fn insert(
        pool: &PgPool,
        request_url: &str,
        request_method: &str,
        request_headers: Option<serde_json::Value>,
        request_body: Option<&str>,
        response_status: Option<i32>,
        response_headers: Option<serde_json::Value>,
        response_body: Option<&str>,
        duration_ms: Option<i32>,
    ) -> anyhow::Result<Uuid> {
        let row = HttpClientLogModel::create(DbConn::pool(pool))
            .set(HttpClientLogCol::REQUEST_URL, request_url.to_string())?
            .set(HttpClientLogCol::REQUEST_METHOD, request_method.to_string())?
            .set(HttpClientLogCol::REQUEST_HEADERS, request_headers)?
            .set(HttpClientLogCol::REQUEST_BODY, request_body.map(str::to_string))?
            .set(HttpClientLogCol::RESPONSE_STATUS, response_status)?
            .set(HttpClientLogCol::RESPONSE_HEADERS, response_headers)?
            .set(HttpClientLogCol::RESPONSE_BODY, response_body.map(str::to_string))?
            .set(HttpClientLogCol::DURATION_MS, duration_ms)?
            .save()
            .await?;

        Ok(row.id)
    }
}
