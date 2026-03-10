use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use core_i18n::t;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::AppError;

/// Recursively trim all JSON string values; convert empty/whitespace-only to null.
pub fn clean_json_strings(value: &mut Value) {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                *value = Value::Null;
            } else if trimmed.len() != s.len() {
                *value = Value::String(trimmed.to_string());
            }
        }
        Value::Array(arr) => arr.iter_mut().for_each(clean_json_strings),
        Value::Object(map) => map.values_mut().for_each(clean_json_strings),
        _ => {}
    }
}

/// Parse request body bytes → clean strings → deserialize into T.
pub async fn clean_and_deserialize<T, S>(req: Request, state: &S) -> Result<T, AppError>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    let bytes = Bytes::from_request(req, state)
        .await
        .map_err(|e| AppError::BadRequest(format!("{}: {}", t("Invalid JSON"), e)))?;
    let mut value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("{}: {}", t("Invalid JSON"), e)))?;
    clean_json_strings(&mut value);
    serde_json::from_value(value)
        .map_err(|e| AppError::BadRequest(format!("{}: {}", t("Invalid JSON"), e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_string_becomes_null() {
        let mut v = json!({"name": ""});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"name": null}));
    }

    #[test]
    fn test_whitespace_only_becomes_null() {
        let mut v = json!({"name": "   "});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"name": null}));
    }

    #[test]
    fn test_trims_whitespace() {
        let mut v = json!({"name": "  hello  "});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"name": "hello"}));
    }

    #[test]
    fn test_nested_objects() {
        let mut v = json!({"user": {"name": "  alice  ", "bio": ""}});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"user": {"name": "alice", "bio": null}}));
    }

    #[test]
    fn test_arrays() {
        let mut v = json!({"tags": ["  foo  ", "", "bar"]});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"tags": ["foo", null, "bar"]}));
    }

    #[test]
    fn test_non_strings_untouched() {
        let mut v = json!({"count": 42, "active": true, "data": null});
        let expected = v.clone();
        clean_json_strings(&mut v);
        assert_eq!(v, expected);
    }

    #[test]
    fn test_no_trim_needed() {
        let mut v = json!({"name": "hello"});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"name": "hello"}));
    }

    #[test]
    fn test_deeply_nested() {
        let mut v = json!({"a": {"b": {"c": "  deep  "}}});
        clean_json_strings(&mut v);
        assert_eq!(v, json!({"a": {"b": {"c": "deep"}}}));
    }

    #[test]
    fn test_mixed_array_of_objects() {
        let mut v = json!([{"name": " x "}, {"name": ""}]);
        clean_json_strings(&mut v);
        assert_eq!(v, json!([{"name": "x"}, {"name": null}]));
    }
}
