use axum::http::HeaderMap;

pub fn parse_timezone_from_headers(headers: &HeaderMap) -> Option<String> {
    for key in ["x-timezone", "timezone"] {
        if let Some(value) = headers.get(key) {
            if let Ok(raw) = value.to_str() {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}
