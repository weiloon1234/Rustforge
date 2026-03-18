fn main() {
    let app_dir = std::path::Path::new("..").join("app");
    let configs_path = app_dir.join("configs.toml");
    let permissions_path = app_dir.join("permissions.toml");
    let models_dir = app_dir.join("models");
    let framework_paths = db_gen::framework_model_source_paths_from_core_db();
    let out_dir = std::path::Path::new("src");

    println!("cargo:rerun-if-changed={}", configs_path.display());
    println!("cargo:rerun-if-changed={}", permissions_path.display());
    println!("cargo:rerun-if-changed={}", models_dir.display());
    for path in &framework_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

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
    if !models_out.exists() {
        std::fs::create_dir_all(&models_out).expect("Failed to create models out");
    }
    db_gen::generate_enums(&schema, &models_out).expect("Failed to gen enums");
    db_gen::generate_models(&schema, &cfgs, &models_out).expect("Failed to gen models");

    let guards_out = out_dir.join("guards");
    if !guards_out.exists() {
        std::fs::create_dir_all(&guards_out).expect("Failed to create guards out");
    }
    db_gen::generate_auth(&cfgs, &schema, &guards_out).expect("Failed to gen auth");
    db_gen::generate_permissions(&permissions, &out_dir.join("permissions.rs"))
        .expect("Failed to gen permissions");

    db_gen::generate_localized(&cfgs.languages, &cfgs, &schema, out_dir)
        .expect("Failed to gen localized");

    let root_lib = out_dir.join("lib.rs");
    let lib_content = "\
#![allow(dead_code)]
// AUTO-GENERATED FILE — DO NOT EDIT
pub mod models;
pub mod guards;
pub mod permissions;
pub mod localized;
pub use localized::*;
pub mod ts_exports;
pub mod generated { pub use crate::*; }
";
    db_gen::write_if_changed(&root_lib, lib_content).expect("Failed to write root lib.rs");
}
