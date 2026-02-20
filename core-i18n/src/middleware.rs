use axum::{
    extract::Request,
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};

/// Middleware to set the current locale scope based on `X-Locale` or `Accept-Language`.
pub async fn locale_middleware(request: Request, next: Next) -> Response {
    let locale = resolve_locale(request.headers()).unwrap_or_else(crate::current_locale);
    crate::scope_locale(locale, || async move { next.run(request).await }).await
}

fn resolve_locale(headers: &HeaderMap) -> Option<&'static str> {
    if let Some(raw) = headers.get("x-locale").and_then(|v| v.to_str().ok()) {
        if let Some(locale) = crate::match_supported_locale(raw) {
            return Some(locale);
        }
    }

    let accept_language = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())?;

    for part in accept_language.split(',') {
        let candidate = part.split(';').next().unwrap_or_default().trim();
        if let Some(locale) = crate::match_supported_locale(candidate) {
            return Some(locale);
        }
    }

    None
}
