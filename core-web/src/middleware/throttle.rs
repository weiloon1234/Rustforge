use std::future::Future;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use governor::{clock::DefaultClock, state::keyed::DashMapStateStore, Quota, RateLimiter};

use crate::error::AppError;

type KeyedLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>;

/// Tower Layer that applies per-IP rate limiting.
///
/// Produces a `ThrottleService<S>` wrapping the inner service.
/// Throttle error returns a 429 response via `AppError::TooManyRequests`.
///
/// Construct with `ThrottleLayer::new(per_second, burst_size)` and apply via
/// `ApiRouter::layer(ThrottleLayer::new(...))`.
#[derive(Clone)]
pub struct ThrottleLayer {
    limiter: Arc<KeyedLimiter>,
}

impl ThrottleLayer {
    /// `per_second`: sustained token refill rate.
    /// `burst_size`: max burst before throttling kicks in.
    pub fn new(per_second: u32, burst_size: u32) -> Self {
        let per_second = per_second.max(1);
        let burst_size = burst_size.max(1);
        let quota = Quota::per_second(NonZeroU32::new(per_second).unwrap())
            .allow_burst(NonZeroU32::new(burst_size).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::dashmap(quota)),
        }
    }
}

impl<S> tower::Layer<S> for ThrottleLayer {
    type Service = ThrottleService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ThrottleService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Wraps an inner service with per-IP rate limiting.
#[derive(Clone)]
pub struct ThrottleService<S> {
    inner: S,
    limiter: Arc<KeyedLimiter>,
}

impl<S> tower::Service<Request<Body>> for ThrottleService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
    S::Error: Into<std::convert::Infallible>,
{
    type Response = Response;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response, std::convert::Infallible>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let ip = extract_ip(&req);

        if self.limiter.check_key(&ip).is_err() {
            let response =
                AppError::TooManyRequests("Rate limit exceeded".to_string()).into_response();
            return Box::pin(async move { Ok(response) });
        }

        let fut = self.inner.call(req);
        Box::pin(async move { fut.await.map_err(Into::into) })
    }
}

/// Extracts the real client IP from request headers / extensions.
/// Priority: X-Real-IP → X-Forwarded-For (first entry) → ConnectInfo peer.
fn extract_ip(req: &Request<Body>) -> IpAddr {
    req.headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next()?.trim().parse().ok())
        })
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip())
        })
        .unwrap_or_else(|| IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
}
