use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FieldInput<T> {
    #[default]
    Unset,
    Set(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldChange<T> {
    Assign(T),
    Increment(T),
    Decrement(T),
}

pub fn renumber_placeholders(sql: &str, start: usize) -> String {
    let mut out = String::with_capacity(sql.len() + 8);
    let mut i = 0;
    let bytes = sql.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'$' {
            i += 1;
            let start_idx = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
            let num: usize = sql[start_idx..i].parse().unwrap_or(0);
            let new_idx = start + num - 1;
            out.push('$');
            out.push_str(&new_idx.to_string());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub per_page: i64,
    pub current_page: i64,
    pub last_page: i64,
}

pub fn log_observer_error(hook: &str, model: &str, err: &dyn std::fmt::Display) {
    eprintln!("model observer hook '{hook}' failed for model '{model}': {err}");
}
