use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde_json::Value;

use crate::{current_locale, locale::match_supported_locale};

type LocaleCatalog = HashMap<String, HashMap<String, String>>;

static TRANSLATIONS: OnceLock<LocaleCatalog> = OnceLock::new();

/// Translate a key using the current request locale.
///
/// English text can be used as key and fallback value.
pub fn t(key: &str) -> String {
    t_for_locale(current_locale(), key)
}

/// Translate a key then replace `:param` placeholders.
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut out = t(key);
    for (k, v) in args {
        out = out.replace(&format!(":{k}"), v);
    }
    out
}

pub fn t_for_locale(locale: &str, key: &str) -> String {
    let Some(locale) = match_supported_locale(locale) else {
        return key.to_string();
    };

    let catalogs = TRANSLATIONS.get_or_init(load_translations);
    catalogs
        .get(locale)
        .and_then(|messages| messages.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

pub fn warmup() {
    let _ = TRANSLATIONS.get_or_init(load_translations);
}

fn load_translations() -> LocaleCatalog {
    let mut catalogs: LocaleCatalog = HashMap::new();
    let mut visited = HashSet::new();

    for dir in translation_dirs() {
        let dir_canonical = std::fs::canonicalize(&dir).unwrap_or(dir.clone());
        if !visited.insert(dir_canonical.clone()) {
            continue;
        }
        load_catalog_from_dir(&dir_canonical, &mut catalogs);
    }

    catalogs
}

fn translation_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();

    if let Ok(custom) = std::env::var("I18N_DIR") {
        let path = PathBuf::from(custom);
        if path.exists() {
            out.push(path);
        }
    }

    if let Ok(mut cursor) = std::env::current_dir() {
        loop {
            let candidate = cursor.join("i18n");
            if candidate.exists() {
                out.push(candidate);
            }

            let Some(parent) = cursor.parent() else {
                break;
            };
            cursor = parent.to_path_buf();
        }
    }

    out
}

fn load_catalog_from_dir(dir: &Path, catalogs: &mut LocaleCatalog) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(locale) = match_supported_locale(stem) else {
            continue;
        };

        let Ok(raw) = std::fs::read_to_string(&path) else {
            tracing::warn!("Failed to read i18n file: {}", path.display());
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<HashMap<String, Value>>(&raw) else {
            tracing::warn!("Failed to parse i18n file: {}", path.display());
            continue;
        };

        let map = parsed
            .into_iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
            .collect::<HashMap<_, _>>();

        catalogs.insert(locale.to_string(), map);
    }
}
