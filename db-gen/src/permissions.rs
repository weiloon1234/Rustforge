use serde::Deserialize;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionEntry {
    pub key: String,
    pub guard: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Default)]
struct PermissionCatalog {
    #[serde(default)]
    permissions: Vec<PermissionEntry>,
    #[serde(default)]
    permission: Vec<PermissionEntry>,
}

pub fn load_permissions(filename: &str) -> Result<Vec<PermissionEntry>, Box<dyn Error>> {
    let check_paths = vec![
        PathBuf::from(filename),
        PathBuf::from("../../").join(filename),
        PathBuf::from("../../../").join(filename),
    ];

    let mut found: Option<PathBuf> = None;
    for path in check_paths {
        if path.exists() {
            found = Some(path);
            break;
        }
    }

    let Some(path) = found else {
        return Ok(Vec::new());
    };

    let raw = fs::read_to_string(path)?;
    let mut catalog: PermissionCatalog = toml::from_str(&raw)?;
    catalog.permissions.append(&mut catalog.permission);

    normalize_and_validate(catalog.permissions)
}

fn normalize_and_validate(
    entries: Vec<PermissionEntry>,
) -> Result<Vec<PermissionEntry>, Box<dyn Error>> {
    let mut out = Vec::with_capacity(entries.len());
    let mut seen_keys = BTreeSet::new();

    for mut entry in entries {
        entry.key = entry.key.trim().to_string();
        entry.guard = entry.guard.trim().to_string();
        entry.label = entry.label.trim().to_string();
        entry.group = entry.group.trim().to_string();
        entry.description = entry.description.trim().to_string();

        if entry.key.is_empty() {
            return Err("permission key must not be empty".into());
        }
        if entry.guard.is_empty() {
            return Err(format!("permission '{}' is missing guard", entry.key).into());
        }
        if !seen_keys.insert(entry.key.clone()) {
            return Err(format!("duplicate permission key '{}'", entry.key).into());
        }

        if entry.label.is_empty() {
            entry.label = entry.key.clone();
        }
        if entry.group.is_empty() {
            entry.group = entry
                .key
                .split('.')
                .next()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("general")
                .to_string();
        }

        out.push(entry);
    }

    Ok(out)
}
