//! Webhook logging middleware for Axum
//!
//! Logs incoming HTTP requests to specified path prefixes.

use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::header::HeaderMap,
    middleware::Next,
    response::Response,
};
use http_body_util::BodyExt;
use sqlx::PgPool;
use std::{sync::Arc, time::Instant};
use tower::{Layer, Service};

use crate::models::WebhookLog;

/// Configuration for webhook logging
#[derive(Clone)]
pub struct WebhookLogConfig {
    pub enabled: bool,
    pub path_prefixes: Vec<String>,
}

impl Default for WebhookLogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path_prefixes: vec!["/wh/".to_string(), "/webhook/".to_string()],
        }
    }
}

/// Layer for webhook logging middleware
#[derive(Clone)]
pub struct WebhookLogLayer {
    pool: PgPool,
    config: WebhookLogConfig,
}

impl WebhookLogLayer {
    pub fn new(pool: PgPool, config: WebhookLogConfig) -> Self {
        Self { pool, config }
    }
}

impl<S> Layer<S> for WebhookLogLayer {
    type Service = WebhookLogMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WebhookLogMiddleware {
            inner,
            pool: self.pool.clone(),
            config: self.config.clone(),
        }
    }
}

/// Middleware service for webhook logging
#[derive(Clone)]
pub struct WebhookLogMiddleware<S> {
    inner: S,
    pool: PgPool,
    config: WebhookLogConfig,
}

impl<S> WebhookLogMiddleware<S> {
    fn should_log(&self, path: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        self.config
            .path_prefixes
            .iter()
            .any(|prefix| path.starts_with(prefix))
    }
}

impl<S> Service<Request> for WebhookLogMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let path = request.uri().path().to_string();

        if !self.should_log(&path) {
            // Not logging this path, pass through
            let future = self.inner.call(request);
            return Box::pin(async move { future.await });
        }

        let pool = self.pool.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start = Instant::now();

            // Extract request info
            let method = request.method().to_string();
            let url = request.uri().to_string();
            let headers = headers_to_json(request.headers());

            // Buffer the request body
            let (parts, body) = request.into_parts();
            let body_bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => Bytes::new(),
            };
            let request_body = String::from_utf8_lossy(&body_bytes).to_string();

            // Reconstruct request with buffered body
            let request = Request::from_parts(parts, Body::from(body_bytes));

            // Call the inner service
            let response = inner.call(request).await?;

            // Extract response info
            let status = response.status().as_u16() as i32;

            // Buffer the response body
            let (parts, body) = response.into_parts();
            let body_bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => Bytes::new(),
            };
            let response_body = String::from_utf8_lossy(&body_bytes).to_string();

            let duration_ms = start.elapsed().as_millis() as i32;

            // Log to database (spawn to not block response)
            let log_pool = pool.clone();
            tokio::spawn(async move {
                if let Err(e) = WebhookLog::insert(
                    &log_pool,
                    &url,
                    &method,
                    Some(headers),
                    Some(&request_body),
                    Some(status),
                    Some(&response_body),
                    Some(duration_ms),
                )
                .await
                {
                    tracing::error!("Failed to log webhook request: {}", e);
                }
            });

            // Reconstruct response with buffered body
            Ok(Response::from_parts(parts, Body::from(body_bytes)))
        })
    }
}

/// Convert headers to JSON
fn headers_to_json(headers: &HeaderMap) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            map.insert(name.to_string(), serde_json::Value::String(v.to_string()));
        }
    }
    serde_json::Value::Object(map)
}

/// Standalone middleware function (alternative to Layer)
pub async fn webhook_log_middleware(
    pool: Arc<PgPool>,
    config: WebhookLogConfig,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Check if we should log this path
    if !config.enabled || !config.path_prefixes.iter().any(|p| path.starts_with(p)) {
        return next.run(request).await;
    }

    let start = Instant::now();

    // Extract request info
    let method = request.method().to_string();
    let url = request.uri().to_string();
    let headers = headers_to_json(request.headers());

    // Buffer the request body
    let (parts, body) = request.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };
    let request_body = String::from_utf8_lossy(&body_bytes).to_string();

    // Reconstruct request
    let request = Request::from_parts(parts, Body::from(body_bytes.clone()));

    // Call next
    let response = next.run(request).await;

    // Extract response info
    let status = response.status().as_u16() as i32;

    // Buffer response body
    let (parts, body) = response.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };
    let response_body = String::from_utf8_lossy(&body_bytes).to_string();

    let duration_ms = start.elapsed().as_millis() as i32;

    // Log to database
    let log_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = WebhookLog::insert(
            &log_pool,
            &url,
            &method,
            Some(headers),
            Some(&request_body),
            Some(status),
            Some(&response_body),
            Some(duration_ms),
        )
        .await
        {
            tracing::error!("Failed to log webhook request: {}", e);
        }
    });

    Response::from_parts(parts, Body::from(body_bytes))
}
