use std::convert::Infallible;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, HeaderMap};

#[derive(Debug, Clone, Default)]
pub struct RequestHeaders(pub HeaderMap);

impl Deref for RequestHeaders {
    type Target = HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequestHeaders
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let headers = parts.headers.clone();
        async move { Ok(Self(headers)) }
    }
}

impl aide::OperationInput for RequestHeaders {}
