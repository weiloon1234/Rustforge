//! Database access wrappers for HTTP traffic logs.

use core_db::{
    common::sql::DbConn,
    generated::models::{HttpClientLog as HttpClientLogModel, WebhookLog as WebhookLogModel},
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
        let row = WebhookLogModel::new(DbConn::pool(pool), None)
            .insert()
            .set_request_url(request_url.to_string())
            .set_request_method(request_method.to_string())
            .set_request_headers(request_headers)
            .set_request_body(request_body.map(str::to_string))
            .set_response_status(response_status)
            .set_response_body(response_body.map(str::to_string))
            .set_duration_ms(duration_ms)
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
        let row = HttpClientLogModel::new(DbConn::pool(pool), None)
            .insert()
            .set_request_url(request_url.to_string())
            .set_request_method(request_method.to_string())
            .set_request_headers(request_headers)
            .set_request_body(request_body.map(str::to_string))
            .set_response_status(response_status)
            .set_response_headers(response_headers)
            .set_response_body(response_body.map(str::to_string))
            .set_duration_ms(duration_ms)
            .save()
            .await?;

        Ok(row.id)
    }
}
