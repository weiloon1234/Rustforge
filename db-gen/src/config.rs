use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ConfigsFile {
    pub languages: Locales,
    #[serde(default, rename = "attachment_type")]
    pub attachment_types: BTreeMap<String, AttachmentType>,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct Locales {
    pub default: String,
    pub supported: Vec<String>,
    /// Optional default timezone in ±HH:MM (fixed offset) form, e.g. "+08:00".
    #[serde(default)]
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[allow(dead_code)]
pub struct AttachmentType {
    pub allowed: Vec<String>,
    #[serde(default)]
    pub resize: Option<Resize>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[allow(dead_code)]
pub struct Resize {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuthConfig {
    pub default: Option<String>,
    #[serde(default)]
    pub guards: BTreeMap<String, GuardConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GuardConfig {
    pub provider: String,
}

use std::path::PathBuf;

pub fn load(filename: &str) -> Result<(ConfigsFile, PathBuf), Box<dyn Error>> {
    let check_paths = vec![
        PathBuf::from(filename),
        PathBuf::from("../../").join(filename),
        PathBuf::from("../../../").join(filename), // Just in case of deeper nesting
    ];

    for path in check_paths {
        if path.exists() {
            // println!(
            //     "cargo:warning=Loading configs from: {:?}",
            //     path.canonicalize().unwrap_or(path.clone())
            // );
            let raw = fs::read_to_string(&path)?;
            let file: ConfigsFile = toml::from_str(&raw)?;
            return Ok((file, path));
        }
    }

    Err(format!("Could not find '{}' in search paths.", filename).into())
}

pub fn validate_locales(l: &Locales) {
    for loc in &l.supported {
        if !is_valid_rust_ident(loc) {
            panic!(
                "Invalid locale '{}'. Use Rust identifiers: en, zh, zh_cn (no '-' like zh-CN).",
                loc
            );
        }
    }
    if !l.supported.iter().any(|s| s == &l.default) {
        panic!(
            "languages.default '{}' must appear in languages.supported (configs.toml)",
            l.default
        );
    }
    if let Some(tz) = &l.timezone {
        validate_timezone(tz);
    }
}

pub fn is_valid_rust_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn validate_timezone(s: &str) {
    // Very small parser for "+HH:MM" / "-HH:MM"
    if s.len() != 6 {
        panic!("Invalid timezone '{}', expected format like +08:00", s);
    }
    let bytes = s.as_bytes();
    if bytes[0] != b'+' && bytes[0] != b'-' {
        panic!("Invalid timezone '{}', must start with + or -", s);
    }
    let hh: i8 = s[1..3].parse().expect("Invalid hour in timezone");
    let mm: i8 = s[4..6].parse().expect("Invalid minute in timezone");
    if hh.abs() > 23 || mm.abs() > 59 {
        panic!(
            "Invalid timezone '{}', hours must be <=23 and minutes <=59",
            s
        );
    }
}
