use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use redis::AsyncCommands;
use serde::Serialize;
use sqlx::FromRow;

use crate::queue::RedisQueue;

#[derive(Serialize)]
pub struct QueueInfo {
    name: String,
    size: u64,
}

#[derive(Clone)]
pub struct ApiState {
    pub queue: RedisQueue,
    pub db: Option<sqlx::PgPool>,
}

#[derive(Serialize, FromRow)]
pub struct FailedJobRow {
    pub id: uuid::Uuid,
    pub job_name: String,
    pub queue: String,
    pub error: String,
    pub attempts: i32,
    pub failed_at: time::OffsetDateTime,
}

pub fn routes(queue: RedisQueue, db: Option<sqlx::PgPool>) -> Router {
    let state = ApiState { queue, db };
    Router::new()
        .route("/", get(list_queues))
        .route("/:name", get(get_queue).delete(clear_queue))
        .route("/:name/jobs", get(peek_jobs))
        .route("/failed", get(list_failed_jobs))
        .route("/failed/:id/retry", post(retry_failed_job))
        .with_state(state)
}

// ... handlers

async fn list_failed_jobs(State(state): State<ApiState>) -> impl IntoResponse {
    let db = match state.db {
        Some(db) => db,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "DB not configured").into_response(),
    };

    let sql = "SELECT id, job_name, queue, error, attempts, failed_at FROM failed_jobs ORDER BY failed_at DESC LIMIT 50";
    let jobs: Vec<FailedJobRow> = match sqlx::query_as(sql).fetch_all(&db).await {
        Ok(rows) => rows,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    Json(jobs).into_response()
}

async fn retry_failed_job(
    State(state): State<ApiState>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = match state.db {
        Some(db) => db,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "DB not configured").into_response(),
    };

    // 1. Fetch the failed job
    let row: Option<(String, serde_json::Value)> =
        match sqlx::query_as("SELECT queue, payload FROM failed_jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(&db)
            .await
        {
            Ok(r) => r,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

    let (queue_name, payload) = match row {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
    };

    let payload_queue = payload
        .get("queue")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or(queue_name);

    // 2. Re-enqueue it into Redis
    let mut conn = match state.queue.client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let queue_key = format!("{}:{}", state.queue.prefix, payload_queue);
    // Convert payload to string
    let payload_str = payload.to_string();

    match conn.rpush::<_, _, ()>(&queue_key, &payload_str).await {
        Ok(_) => (),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }

    // 3. Delete from failed_jobs
    match sqlx::query("DELETE FROM failed_jobs WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
    {
        Ok(_) => (StatusCode::OK, "Retried").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn list_queues(State(state): State<ApiState>) -> impl IntoResponse {
    // Scan for keys starting with prefix
    let mut conn = match state.queue.client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let pattern = format!("{}:*", state.queue.prefix);
    let mut cursor: u64 = 0;
    let mut keys = Vec::new();
    loop {
        let (next, batch): (u64, Vec<String>) = match redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(200)
            .query_async(&mut conn)
            .await
        {
            Ok(r) => r,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

        keys.extend(batch);
        if next == 0 {
            break;
        }
        cursor = next;
    }

    let mut queues = Vec::new();
    for k in keys {
        // filter out meta/locks? keep all for visibility
        let size: u64 = conn.llen(&k).await.unwrap_or(0);
        queues.push(QueueInfo { name: k, size });
    }

    Json(queues).into_response()
}

async fn get_queue(State(state): State<ApiState>, Path(name): Path<String>) -> impl IntoResponse {
    // Determine fully qualified key?
    // User passes "default". We assume "{prefix}:{name}"?
    // Or users pass full key from list_queues?
    // Let's assume list_queues returns full keys (e.g. "queue:default").
    // So name is full key.
    // Note: URL encoding issues if name contains ":".
    // Better: Helper API exposes logical names?
    // For raw control, we might accept base64 encoded key or just the string if router allows.
    // Simple approach: name is the suffix.

    let key = format!("{}:{}", state.queue.prefix, name);
    let mut conn = match state.queue.client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let size: u64 = conn.llen(&key).await.unwrap_or(0);

    Json(QueueInfo { name: key, size }).into_response()
}

async fn clear_queue(State(state): State<ApiState>, Path(name): Path<String>) -> impl IntoResponse {
    let key = format!("{}:{}", state.queue.prefix, name);
    let mut conn = match state.queue.client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let _: () = conn.del(&key).await.unwrap_or(());
    (StatusCode::OK, "Queue cleared").into_response()
}

async fn peek_jobs(State(state): State<ApiState>, Path(name): Path<String>) -> impl IntoResponse {
    let key = format!("{}:{}", state.queue.prefix, name);
    let mut conn = match state.queue.client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    // Peek top 10
    let jobs: Vec<String> = conn.lrange(&key, 0, 10).await.unwrap_or_default();

    // Attempt to parse JSON? Or return raw strings.
    let parsed: Vec<serde_json::Value> = jobs
        .iter()
        .map(|s| serde_json::from_str(s).unwrap_or(serde_json::json!({"raw": s})))
        .collect();

    Json(parsed).into_response()
}

// Handlers replaced by block above
