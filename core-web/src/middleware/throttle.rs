use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{extract::Request, middleware::Next, response::Response};
use governor::{clock::DefaultClock, state::keyed::DashMapStateStore, Quota, RateLimiter};

use crate::error::AppError;

type KeyedLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock>;

/// Returns an axum middleware layer that throttles requests per client IP.
///
/// `per_second`: sustained token refill rate (requests allowed per second).
/// `burst_size`: maximum burst of requests before throttling kicks in.
///
/// IP is resolved from X-Real-IP → X-Forwarded-For → ConnectInfo peer, in that order.
/// This works correctly behind nginx (which sets X-Real-IP to the real client IP).
pub fn throttle_layer(
    per_second: u32,
    burst_size: u32,
) -> axum::middleware::FromFn<
    impl Fn(
            axum::extract::Request,
            axum::middleware::Next,
        ) -> impl std::future::Future<Output = Result<Response, AppError>>
        + Clone
        + Send
        + 'static,
> {
    let per_second = per_second.max(1);
    let burst_size = burst_size.max(1);

    let quota = Quota::per_second(NonZeroU32::new(per_second).unwrap())
        .allow_burst(NonZeroU32::new(burst_size).unwrap());

    let limiter: Arc<KeyedLimiter> = Arc::new(RateLimiter::dashmap(quota));

    axum::middleware::from_fn(move |req: Request, next: Next| {
        let limiter = limiter.clone();
        async move {
            let ip = extract_ip(&req);
            match limiter.check_key(&ip) {
                Ok(_) => Ok(next.run(req).await),
                Err(_) => Err(AppError::TooManyRequests("Rate limit exceeded".to_string())),
            }
        }
    })
}

/// Extracts the real client IP from request headers / extensions.
/// Priority: X-Real-IP → X-Forwarded-For (first entry) → ConnectInfo peer.
fn extract_ip(req: &Request) -> IpAddr {
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
