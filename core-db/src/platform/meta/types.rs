#![allow(dead_code)]
use std::collections::HashMap;

/// field -> owner_id -> value
#[derive(Debug, Clone, Default)]
pub struct MetaMap {
    inner: HashMap<String, HashMap<i64, serde_json::Value>>,
}

impl MetaMap {
    pub fn new(inner: HashMap<String, HashMap<i64, serde_json::Value>>) -> Self {
        Self { inner }
    }

    pub fn get_value(&self, field: &str, owner_id: i64) -> Option<serde_json::Value> {
        self.inner.get(field)?.get(&owner_id).cloned()
    }

    /// Returns a cloned map of all meta for one owner (empty if none).
    pub fn get_all_for_owner(&self, owner_id: i64) -> HashMap<String, serde_json::Value> {
        let mut out = HashMap::new();
        for (field, by_owner) in &self.inner {
            if let Some(v) = by_owner.get(&owner_id) {
                out.insert(field.clone(), v.clone());
            }
        }
        out
    }
}
