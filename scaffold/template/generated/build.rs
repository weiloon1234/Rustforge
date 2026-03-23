use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn main() {
    let app_dir = std::path::Path::new("..").join("app");
    let configs_path = app_dir.join("configs.toml");
    let permissions_path = app_dir.join("permissions.toml");
    let models_dir = app_dir.join("models");
    let framework_paths = db_gen::framework_model_source_paths_from_core_db();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    // Watch individual files, not the directory — avoids spurious triggers from
    // editor temp files, .DS_Store, etc.
    println!("cargo:rerun-if-changed={}", configs_path.display());
    println!("cargo:rerun-if-changed={}", permissions_path.display());
    if models_dir.is_dir() {
        for entry in std::fs::read_dir(&models_dir).expect("Failed to read models dir") {
            let entry = entry.expect("Failed to read model entry");
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "rs") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    for path in &framework_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    // Content-hash guard: skip generation if inputs haven't changed.
    let input_hash = hash_inputs(&configs_path, &permissions_path, &models_dir, &framework_paths);
    let hash_file = out_dir.join(".generation_hash");
    if hash_file.exists() {
        if let Ok(existing) = std::fs::read_to_string(&hash_file) {
            if existing.trim() == input_hash {
                return;
            }
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

    // Generate the include root — same pattern as core-db/build.rs
    use std::fmt::Write as _;
    let mut root = String::new();
    let escape = |p: PathBuf| p.display().to_string().replace('\\', "\\\\");

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

fn hash_inputs(
    configs_path: &std::path::Path,
    permissions_path: &std::path::Path,
    models_dir: &std::path::Path,
    framework_paths: &[PathBuf],
) -> String {
    let mut hasher = DefaultHasher::new();

    // Hash file contents, not just metadata — actual content changes matter.
    if let Ok(data) = std::fs::read(configs_path) {
        data.hash(&mut hasher);
    }
    if let Ok(data) = std::fs::read(permissions_path) {
        data.hash(&mut hasher);
    }
    if models_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(models_dir)
            .expect("Failed to read models dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            entry.file_name().hash(&mut hasher);
            if let Ok(data) = std::fs::read(entry.path()) {
                data.hash(&mut hasher);
            }
        }
    }
    for path in framework_paths {
        if let Ok(data) = std::fs::read(path) {
            data.hash(&mut hasher);
        }
    }

    format!("{:x}", hasher.finish())
}
