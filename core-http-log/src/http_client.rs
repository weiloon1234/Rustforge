//! HTTP Client with logging interceptors
//!
//! A wrapper around `reqwest::Client` that logs all outbound HTTP requests.

use reqwest::{header::HeaderMap, Method, Response};
use serde::Serialize;
use sqlx::PgPool;
use std::{sync::Arc, time::Instant};

use crate::models::HttpClientLog;

/// Configuration for HTTP client logging
#[derive(Clone)]
pub struct HttpClientConfig {
    pub enabled: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

/// HTTP Client with automatic request/response logging
#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    pool: Option<Arc<PgPool>>,
    config: HttpClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client with logging
    pub fn new(pool: &PgPool, config: HttpClientConfig) -> Self {
        Self {
            inner: reqwest::Client::new(),
            pool: Some(Arc::new(pool.clone())),
            config,
        }
    }

    /// Create a new HTTP client without logging (for use when DB is not available)
    pub fn new_without_logging() -> Self {
        Self {
            inner: reqwest::Client::new(),
            pool: None,
            config: HttpClientConfig { enabled: false },
        }
    }

    /// Create a new HTTP client with a custom reqwest client
    pub fn with_client(client: reqwest::Client, pool: &PgPool, config: HttpClientConfig) -> Self {
        Self {
            inner: client,
            pool: Some(Arc::new(pool.clone())),
            config,
        }
    }

    /// Get the inner reqwest client (for direct access when needed)
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Create a GET request builder
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.clone(), Method::GET, url.to_string())
    }

    /// Create a POST request builder
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.clone(), Method::POST, url.to_string())
    }

    /// Create a PUT request builder
    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.clone(), Method::PUT, url.to_string())
    }

    /// Create a PATCH request builder
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.clone(), Method::PATCH, url.to_string())
    }

    /// Create a DELETE request builder
    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.clone(), Method::DELETE, url.to_string())
    }

    /// Check if logging is enabled
    fn should_log(&self) -> bool {
        self.config.enabled && self.pool.is_some()
    }

    /// Log a request/response to the database
    async fn log_request(
        &self,
        url: &str,
        method: &str,
        request_headers: Option<serde_json::Value>,
        request_body: Option<&str>,
        response_status: Option<i32>,
        response_headers: Option<serde_json::Value>,
        response_body: Option<&str>,
        duration_ms: i32,
    ) {
        if let Some(ref pool) = self.pool {
            if let Err(e) = HttpClientLog::insert(
                pool,
                url,
                method,
                request_headers,
                request_body,
                response_status,
                response_headers,
                response_body,
                Some(duration_ms),
            )
            .await
            {
                tracing::error!("Failed to log HTTP client request: {}", e);
            }
        }
    }
}

/// Request builder with logging support
pub struct RequestBuilder {
    client: HttpClient,
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Option<String>,
    json_body: Option<serde_json::Value>,
}

impl RequestBuilder {
    fn new(client: HttpClient, method: Method, url: String) -> Self {
        Self {
            client,
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            json_body: None,
        }
    }

    /// Add a header
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            key.parse::<reqwest::header::HeaderName>(),
            value.parse::<reqwest::header::HeaderValue>(),
        ) {
            self.headers.insert(name, val);
        }
        self
    }

    /// Add multiple headers
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Set the request body as raw string
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the request body as JSON
    pub fn json<T: Serialize>(mut self, json: &T) -> Self {
        if let Ok(value) = serde_json::to_value(json) {
            self.json_body = Some(value);
        }
        self
    }

    /// Set bearer token authorization
    pub fn bearer_auth(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {}", token))
    }

    /// Set basic authorization
    pub fn basic_auth(self, username: &str, password: Option<&str>) -> Self {
        use base64::Engine;
        let credentials = match password {
            Some(p) => format!("{}:{}", username, p),
            None => format!("{}:", username),
        };
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        self.header("Authorization", &format!("Basic {}", encoded))
    }

    /// Send the request
    pub async fn send(self) -> Result<Response, reqwest::Error> {
        let start = Instant::now();

        // Build the request
        let mut req = self.client.inner.request(self.method.clone(), &self.url);

        // Add headers
        for (key, value) in self.headers.iter() {
            req = req.header(key, value);
        }

        // Prepare body for logging
        let body_for_log: Option<String>;

        // Set body
        if let Some(json) = &self.json_body {
            body_for_log = Some(json.to_string());
            req = req.json(json);
        } else if let Some(body) = &self.body {
            body_for_log = Some(body.clone());
            req = req.body(body.clone());
        } else {
            body_for_log = None;
        }

        // Execute request
        let response = req.send().await?;

        // Log if enabled
        if self.client.should_log() {
            let duration_ms = start.elapsed().as_millis() as i32;
            let status = response.status().as_u16() as i32;

            // Convert headers to JSON
            let request_headers = headers_to_json(&self.headers);
            let response_headers = headers_to_json(response.headers());

            // We can't easily get the response body without consuming it,
            // so we log without response body for now (or clone if needed)
            let client = self.client.clone();
            let url = self.url.clone();
            let method = self.method.to_string();

            tokio::spawn(async move {
                client
                    .log_request(
                        &url,
                        &method,
                        Some(request_headers),
                        body_for_log.as_deref(),
                        Some(status),
                        Some(response_headers),
                        None, // Response body not captured to avoid consuming it
                        duration_ms,
                    )
                    .await;
            });
        }

        Ok(response)
    }

    /// Send the request and get the response body as text
    /// This method also logs the response body
    pub async fn send_and_read_text(self) -> Result<(u16, String), reqwest::Error> {
        let start = Instant::now();

        // Build the request
        let mut req = self.client.inner.request(self.method.clone(), &self.url);

        // Add headers
        for (key, value) in self.headers.iter() {
            req = req.header(key, value);
        }

        // Prepare body for logging
        let body_for_log: Option<String>;

        // Set body
        if let Some(json) = &self.json_body {
            body_for_log = Some(json.to_string());
            req = req.json(json);
        } else if let Some(body) = &self.body {
            body_for_log = Some(body.clone());
            req = req.body(body.clone());
        } else {
            body_for_log = None;
        }

        // Execute request
        let response = req.send().await?;
        let status = response.status().as_u16();
        let response_headers_json = headers_to_json(response.headers());
        let response_body = response.text().await?;

        // Log if enabled
        if self.client.should_log() {
            let duration_ms = start.elapsed().as_millis() as i32;
            let request_headers = headers_to_json(&self.headers);

            let client = self.client.clone();
            let url = self.url.clone();
            let method = self.method.to_string();
            let resp_body = response_body.clone();

            tokio::spawn(async move {
                client
                    .log_request(
                        &url,
                        &method,
                        Some(request_headers),
                        body_for_log.as_deref(),
                        Some(status as i32),
                        Some(response_headers_json),
                        Some(&resp_body),
                        duration_ms,
                    )
                    .await;
            });
        }

        Ok((status, response_body))
    }

    /// Send the request and deserialize the response as JSON
    pub async fn send_json<T: serde::de::DeserializeOwned>(self) -> Result<T, HttpClientError> {
        let (status, body) = self.send_and_read_text().await?;
        if status >= 200 && status < 300 {
            serde_json::from_str(&body).map_err(HttpClientError::Json)
        } else {
            Err(HttpClientError::HttpError { status, body })
        }
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

/// HTTP Client errors
#[derive(Debug)]
pub enum HttpClientError {
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    HttpError { status: u16, body: String },
}

impl std::fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "HTTP request error: {}", e),
            Self::Json(e) => write!(f, "JSON parse error: {}", e),
            Self::HttpError { status, body } => {
                write!(f, "HTTP error {}: {}", status, body)
            }
        }
    }
}

impl std::error::Error for HttpClientError {}

impl From<reqwest::Error> for HttpClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}
