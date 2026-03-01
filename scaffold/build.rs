use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=template");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let template_dir = manifest_dir.join("template");
    ensure_no_generated_dirs(&template_dir);
}

fn ensure_no_generated_dirs(root: &Path) {
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

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if forbidden.iter().any(|f| f == &name) {
                panic!(
                    "found forbidden generated directory in scaffold templates: {}\n\
                     remove it before building scaffold (embedded template must be source-only)",
                    path.display()
                );
            }

            stack.push(path);
        }
    }
}

