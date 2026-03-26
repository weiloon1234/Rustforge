use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn main() {
    let app_dir = std::path::Path::new("..").join("app");
    let configs_path = app_dir.join("settings.toml");
    let permissions_path = app_dir.join("permissions.toml");
    let models_dir = app_dir.join("models");
    let framework_paths = db_gen::framework_model_source_paths_from_core_db();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    let model_files = collect_rs_files(&models_dir);

    // Watch individual files, not the directory — avoids spurious triggers from
    // editor temp files, .DS_Store, etc.
    println!("cargo:rerun-if-changed={}", configs_path.display());
    println!("cargo:rerun-if-changed={}", permissions_path.display());
    for path in &model_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for path in &framework_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    // Content-hash guard: skip generation if inputs haven't changed.
    let input_hash = hash_inputs(&configs_path, &permissions_path, &model_files, &framework_paths);
    let hash_file = out_dir.join(".generation_hash");
    if let Ok(existing) = std::fs::read_to_string(&hash_file) {
        if existing.trim() == input_hash {
            return;
        }
    }

    let (cfgs, _) =
        db_gen::config::load(configs_path.to_str().unwrap()).expect("Failed to load configs");

    let schema = db_gen::load_with_framework_from_paths(
        models_dir.to_str().unwrap(),
        &framework_paths,
    )
    .expect("Failed to load layered models");
    let permissions = db_gen::load_permissions(permissions_path.to_str().unwrap())
        .expect("Failed to load permissions");

    let models_out = out_dir.join("models");
    std::fs::create_dir_all(&models_out).expect("Failed to create models out");
    db_gen::generate_enums(&schema, &models_out).expect("Failed to gen enums");
    db_gen::generate_models(&schema, &cfgs, &models_out).expect("Failed to gen models");

    let guards_out = out_dir.join("guards");
    std::fs::create_dir_all(&guards_out).expect("Failed to create guards out");
    db_gen::generate_auth(&cfgs, &schema, &guards_out).expect("Failed to gen auth");
    db_gen::generate_permissions(&permissions, &out_dir.join("permissions.rs"))
        .expect("Failed to gen permissions");

    db_gen::generate_localized(&cfgs.languages, &cfgs, &schema, &out_dir)
        .expect("Failed to gen localized");

    use std::fmt::Write as _;
    let mut root = String::new();
    let escape = |p: PathBuf| db_gen::escape_path_for_include(&p);

    let _ = writeln!(root, "#[path = \"{}\"] pub mod models;", escape(models_out.join("mod.rs")));
    let _ = writeln!(root, "#[path = \"{}\"] pub mod guards;", escape(guards_out.join("mod.rs")));
    let _ = writeln!(root, "#[path = \"{}\"] pub mod permissions;", escape(out_dir.join("permissions.rs")));
    let _ = writeln!(root, "#[path = \"{}\"] pub mod localized;", escape(out_dir.join("localized.rs")));
    let _ = writeln!(root, "pub use localized::*;");

    db_gen::write_if_changed(&out_dir.join("generated_root.rs"), &root)
        .expect("Failed to write generated root");

    // Persist hash so next build can skip if inputs are unchanged.
    std::fs::write(&hash_file, &input_hash).expect("Failed to write generation hash");
}

/// Collect sorted `.rs` files from a directory (empty vec if dir doesn't exist).
fn collect_rs_files(dir: &std::path::Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return vec![];
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("Failed to read models dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "rs"))
        .collect();
    paths.sort();
    paths
}

fn hash_inputs(
    configs_path: &std::path::Path,
    permissions_path: &std::path::Path,
    model_files: &[PathBuf],
    framework_paths: &[PathBuf],
) -> String {
    let mut hasher = DefaultHasher::new();

    std::fs::read(configs_path)
        .expect("Failed to read settings.toml")
        .hash(&mut hasher);
    std::fs::read(permissions_path)
        .expect("Failed to read permissions.toml")
        .hash(&mut hasher);
    for path in model_files {
        path.file_name().hash(&mut hasher);
        std::fs::read(path)
            .expect("Failed to read model file")
            .hash(&mut hasher);
    }
    for path in framework_paths {
        if let Ok(data) = std::fs::read(path) {
            data.hash(&mut hasher);
        }
    }

    format!("{:x}", hasher.finish())
}
