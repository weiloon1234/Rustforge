//! Database models for HTTP traffic logs

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// Webhook (inbound) request log
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookLog {
    pub id: Uuid,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: OffsetDateTime,
}

impl WebhookLog {
    /// Insert a new webhook log entry
    pub async fn insert(
        pool: &PgPool,
        request_url: &str,
        request_method: &str,
        request_headers: Option<serde_json::Value>,
        request_body: Option<&str>,
        response_status: Option<i32>,
        response_body: Option<&str>,
        duration_ms: Option<i32>,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO webhook_logs (id, request_url, request_method, request_headers, request_body, response_status, response_body, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(request_url)
        .bind(request_method)
        .bind(request_headers)
        .bind(request_body)
        .bind(response_status)
        .bind(response_body)
        .bind(duration_ms)
        .execute(pool)
        .await?;

        Ok(id)
    }
}

/// HTTP Client (outbound) request log
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HttpClientLog {
    pub id: Uuid,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_headers: Option<serde_json::Value>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: OffsetDateTime,
}

impl HttpClientLog {
    /// Insert a new HTTP client log entry
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
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO http_client_logs (id, request_url, request_method, request_headers, request_body, response_status, response_headers, response_body, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(request_url)
        .bind(request_method)
        .bind(request_headers)
        .bind(request_body)
        .bind(response_status)
        .bind(response_headers)
        .bind(response_body)
        .bind(duration_ms)
        .execute(pool)
        .await?;

        Ok(id)
    }
}
