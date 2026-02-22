use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
};
use axum_client_ip::ClientIpSource;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::Span;

use core_config::Settings;

use crate::error::AppError;
use axum::response::IntoResponse;

// Standard middleware stack
pub fn apply_standard_middleware(router: axum::Router, settings: &Settings) -> axum::Router {
    router.layer(
        ServiceBuilder::new()
            .layer(CatchPanicLayer::custom(handle_panic))
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        tracing::info_span!(
                            "request",
                            method = %request.method(),
                            uri = %request.uri(),
                        )
                    })
                    .on_request(|request: &Request<Body>, _span: &Span| {
                        tracing::info!("started {} {}", request.method(), request.uri().path())
                    }),
            )
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(settings.middleware.timeout_secs),
            ))
            .layer(ClientIpSource::ConnectInfo.into_extension())
            .layer(build_cors_layer(settings))
            .layer(DefaultBodyLimit::max(
                settings.middleware.body_limit_mb * 1024 * 1024,
            ))
            .layer(CompressionLayer::new())
            .layer(CookieManagerLayer::new())
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::header::STRICT_TRANSPORT_SECURITY,
                axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                axum::http::HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::header::X_FRAME_OPTIONS,
                axum::http::HeaderValue::from_static("DENY"),
            )),
    )
}

fn build_cors_layer(settings: &Settings) -> CorsLayer {
    use axum::http::Method;

    let cors = &settings.cors;
    let mut layer = CorsLayer::new();

    // Origins
    if cors.allowed_origins.len() == 1 && cors.allowed_origins[0] == "*" {
        layer = layer.allow_origin(Any);
    } else {
        let parsed: Vec<axum::http::HeaderValue> = cors
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        layer = layer.allow_origin(parsed);
    }

    // Methods
    if cors.allowed_methods.len() == 1 && cors.allowed_methods[0] == "*" {
        layer = layer.allow_methods(Any);
    } else {
        let methods: Vec<Method> = cors
            .allowed_methods
            .iter()
            .filter_map(|m| m.parse().ok())
            .collect();
        layer = layer.allow_methods(methods);
    }

    // Headers
    if cors.allowed_headers.len() == 1 && cors.allowed_headers[0] == "*" {
        layer = layer.allow_headers(Any);
    } else {
        let headers: Vec<axum::http::HeaderName> = cors
            .allowed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        layer = layer.allow_headers(headers);
    }

    // Exposed headers
    if !cors.exposed_headers.is_empty() {
        let exposed: Vec<axum::http::HeaderName> = cors
            .exposed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        layer = layer.expose_headers(exposed);
    }

    // Max age
    if cors.max_age > 0 {
        layer = layer.max_age(Duration::from_secs(cors.max_age));
    }

    // Credentials
    if cors.supports_credentials {
        layer = layer.allow_credentials(true);
    }

    layer
}

fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> axum::response::Response {
    let details = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };

    tracing::error!("Panic caught: {}", details);

    AppError::Internal(anyhow::anyhow!("Server panic: {}", details)).into_response()
}
