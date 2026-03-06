use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED_ALLOW_LINTS: &[&str] = &[
    "dead_code",
    "unused_imports",
    "unused_variables",
    "unused_mut",
];
const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "vendor"];

#[test]
fn rust_allow_suppressions_are_limited_to_unused_family() {
    let repo_root = repo_root();
    let mut rs_files = Vec::new();
    collect_files_with_ext(&repo_root, "rs", &mut rs_files);

    let mut violations = Vec::new();
    for file in rs_files {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("#[allow(") || trimmed.starts_with("#![allow(")) {
                continue;
            }

            let Some(start) = trimmed.find("allow(") else {
                continue;
            };
            let after = &trimmed[start + "allow(".len()..];
            let Some(end) = after.find(')') else {
                continue;
            };
            let inner = &after[..end];

            for lint in inner
                .split(',')
                .map(|part| part.trim())
                .filter(|part| !part.is_empty())
            {
                if !ALLOWED_ALLOW_LINTS.contains(&lint) {
                    violations.push(format!(
                        "{}:{} -> disallowed allow lint `{}`",
                        file.display(),
                        idx + 1,
                        lint
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "disallowed allow suppressions found:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ts_rs_warning_suppression_feature_is_not_used() {
    let repo_root = repo_root();
    let mut cargo_manifests = Vec::new();
    collect_named_files(&repo_root, "Cargo.toml", &mut cargo_manifests);

    let mut violations = Vec::new();
    for manifest in cargo_manifests {
        let Ok(content) = fs::read_to_string(&manifest) else {
            continue;
        };
        if content.contains("no-serde-warnings") {
            violations.push(manifest.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "found disallowed ts-rs suppression feature `no-serde-warnings` in:\n{}",
        violations.join("\n")
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("scaffold crate should have parent directory")
        .to_path_buf()
}

fn collect_files_with_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            let skip = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| SKIP_DIRS.contains(&name))
                .unwrap_or(false);
            if !skip {
                collect_files_with_ext(&path, ext, out);
            }
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn collect_named_files(dir: &Path, file_name: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            let skip = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| SKIP_DIRS.contains(&name))
                .unwrap_or(false);
            if !skip {
                collect_named_files(&path, file_name, out);
            }
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) == Some(file_name) {
            out.push(path);
        }
    }
}
