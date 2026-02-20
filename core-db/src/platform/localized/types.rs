#![allow(dead_code)]
use std::collections::HashMap;

const DEFAULT_LOCALE: &str = "en";

#[derive(Debug, Clone, Default)]
pub struct LocalizedMap {
    // field -> owner_id -> locale -> value
    pub inner: HashMap<String, HashMap<i64, HashMap<String, String>>>,
}

impl LocalizedMap {
    pub fn new(inner: HashMap<String, HashMap<i64, HashMap<String, String>>>) -> Self {
        Self { inner }
    }

    /// Get one localized value for (field, owner_id, locale).
    /// Falls back to DEFAULT_LOCALE if the requested locale is missing.
    pub fn get_value(&self, field: &str, owner_id: i64, locale: &str) -> Option<String> {
        let by_owner = self.inner.get(field)?;
        let by_locale = by_owner.get(&owner_id)?;

        if let Some(v) = by_locale.get(locale) {
            return Some(v.clone());
        }

        by_locale.get(DEFAULT_LOCALE).cloned()
    }

    /// Resolved string for locale (fallbacks to "en").
    pub fn get_string(&self, field: &str, owner_id: i64, locale: &str) -> Option<String> {
        self.get_value(field, owner_id, locale)
    }
}
