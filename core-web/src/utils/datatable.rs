use axum::http::HeaderMap;
use core_datatable::DataTableInput;
use serde_json::Value;
use std::collections::BTreeMap;

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

pub fn datatable_input_from_query(params: &BTreeMap<String, String>) -> DataTableInput {
    DataTableInput::from_pairs(params.iter().map(|(k, v)| (k.clone(), v.clone())))
}

pub fn datatable_input_from_form(params: &BTreeMap<String, String>) -> DataTableInput {
    DataTableInput::from_pairs(params.iter().map(|(k, v)| (k.clone(), v.clone())))
}

pub fn datatable_input_from_json(body: &Value) -> anyhow::Result<DataTableInput> {
    let Some(obj) = body.as_object() else {
        anyhow::bail!("datatable json payload must be an object");
    };

    let mut params = BTreeMap::new();
    for (k, v) in obj {
        let value = match v {
            Value::Null => String::new(),
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        };
        params.insert(k.clone(), value);
    }

    Ok(DataTableInput::from_pairs(params))
}
