use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=template");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let template_dir = manifest_dir.join("template");
    validate_template_is_clean(&template_dir);
}

fn validate_template_is_clean(root: &Path) {
    let mut issues = Vec::new();
    collect_forbidden_dirs(root, &mut issues);
    collect_generated_public_entries(&root.join("public"), &mut issues);

    if issues.is_empty() {
        return;
    }

    let mut message = String::from(
        "scaffold/template contains generated artifacts and cannot be packaged as source.\n\
         Remove them explicitly, then rebuild.\n\
         Suggested command: make scaffold-template-clean\n\nDetected paths:\n",
    );
    for issue in issues {
        message.push_str(" - ");
        message.push_str(&issue);
        message.push('\n');
    }
    panic!("{message}");
}

fn collect_forbidden_dirs(root: &Path, issues: &mut Vec<String>) {
    let forbidden = ["target", "node_modules", ".next", "dist"];
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if forbidden.iter().any(|f| f == &name) {
                issues.push(path.display().to_string());
                continue;
            }

            stack.push(path);
        }
    }
}

fn collect_generated_public_entries(public_dir: &Path, issues: &mut Vec<String>) {
    if !public_dir.is_dir() {
        return;
    }

    let mut stack = vec![public_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                issues.push(path.display().to_string());
                stack.push(path);
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name == ".gitkeep" {
                continue;
            }

            issues.push(path.display().to_string());
        }
    }
}
