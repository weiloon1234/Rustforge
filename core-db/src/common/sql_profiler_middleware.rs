use std::sync::Arc;

use axum::{extract::Request, middleware::Next, response::Response};
use sqlx::PgPool;

use super::sql::{is_sql_profiler_enabled, SqlProfilerCollector, PROFILER_COLLECTOR};
use crate::platform::sql_profiler::service::flush_profiler;

/// Axum middleware that wraps each request in a profiler collector scope.
/// When the request completes, collected queries are flushed to the database
/// in a background task.
pub async fn sql_profiler_layer(pool: Arc<PgPool>, request: Request, next: Next) -> Response {
    if !is_sql_profiler_enabled() {
        return next.run(request).await;
    }

    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let collector = Arc::new(SqlProfilerCollector::new());
    let collector_clone = collector.clone();

    let response = PROFILER_COLLECTOR
        .scope(collector_clone, next.run(request))
        .await;

    // Flush in background — don't block the response
    let pool_ref = (*pool).clone();
    tokio::spawn(async move {
        let c = Arc::try_unwrap(collector).unwrap_or_else(|arc| (*arc).clone());
        if let Err(e) = flush_profiler(&pool_ref, &method, &path, c).await {
            tracing::warn!("Failed to flush SQL profiler data: {}", e);
        }
    });

    response
}
