use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let framework_paths = db_gen::framework_model_source_paths_from_core_db();
    for path in &framework_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let schema = db_gen::load_framework_from_paths(&framework_paths)
        .expect("failed to load framework model sources");

    let cfgs = db_gen::config::ConfigsFile {
        languages: db_gen::config::Locales {
            default: "en".to_string(),
            supported: vec!["en".to_string()],
            timezone: None,
        },
        attachment_types: BTreeMap::new(),
        auth: db_gen::config::AuthConfig::default(),
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let models_dir = out_dir.join("framework_generated/models");
    fs::create_dir_all(&models_dir).expect("failed to create models output directory");

    db_gen::generate_enums_with_options(
        &schema,
        &models_dir,
        db_gen::GenerateEnumsOptions {
            include_datatable_filter_options: false,
        },
    )
    .expect("failed to generate framework enums");
    db_gen::generate_models_with_options(
        &schema,
        &cfgs,
        &models_dir,
        db_gen::GenerateModelsOptions {
            include_datatable: false,
            include_extensions_imports: false,
        },
    )
    .expect("failed to generate framework models");

    let mut out = String::new();
    let models_mod_path = db_gen::escape_path_for_include(&models_dir.join("mod.rs"));
    let _ = writeln!(out, "#[path = \"{models_mod_path}\"] pub mod models;");

    fs::write(out_dir.join("framework_generated.rs"), out)
        .expect("failed to write framework generated root include");
}
