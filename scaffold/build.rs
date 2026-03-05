use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=template");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let template_dir = manifest_dir.join("template");
    cleanup_generated_dirs(&template_dir);
    cleanup_public_generated_assets(&template_dir);
}

fn cleanup_generated_dirs(root: &Path) {
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
                fs::remove_dir_all(&path).unwrap_or_else(|error| {
                    panic!(
                        "failed to remove generated scaffold template directory {}: {}",
                        path.display(),
                        error
                    )
                });
                println!(
                    "cargo:warning=Removed generated scaffold template directory: {}",
                    path.display()
                );
                continue;
            }

            stack.push(path);
        }
    }
}

fn cleanup_public_generated_assets(root: &Path) {
    let public_dir = root.join("public");
    if !public_dir.is_dir() {
        return;
    }

    let mut stack = vec![public_dir.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
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

            fs::remove_file(&path).unwrap_or_else(|error| {
                panic!(
                    "failed to remove generated scaffold public asset {}: {}",
                    path.display(),
                    error
                )
            });
            println!(
                "cargo:warning=Removed generated scaffold public asset: {}",
                path.display()
            );
        }
    }

    cleanup_empty_public_dirs(&public_dir);
}

fn cleanup_empty_public_dirs(dir: &Path) -> bool {
    let mut is_empty = true;

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let child_empty = cleanup_empty_public_dirs(&path);
            if child_empty {
                fs::remove_dir(&path).unwrap_or_else(|error| {
                    panic!(
                        "failed to remove empty generated scaffold public directory {}: {}",
                        path.display(),
                        error
                    )
                });
                println!(
                    "cargo:warning=Removed empty generated scaffold public directory: {}",
                    path.display()
                );
            } else {
                is_empty = false;
            }
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name != ".gitkeep" {
            is_empty = false;
        }
    }

    is_empty
}
