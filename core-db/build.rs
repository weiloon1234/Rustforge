use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let schema = db_gen::load_framework().expect("failed to load embedded framework schemas");

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

    let mut modules = fs::read_dir(&models_dir)
        .expect("failed to read generated model directory")
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("failed to collect generated model entries")
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.to_string())
        })
        .filter(|stem| stem != "mod")
        .collect::<Vec<_>>();

    modules.sort();

    let mut out = String::new();
    out.push_str("pub mod models {\n");
    for module in &modules {
        let module_path = models_dir
            .join(format!("{module}.rs"))
            .display()
            .to_string()
            .replace('\\', "\\\\");
        let _ = writeln!(
            out,
            "    #[path = \"{module_path}\"] pub mod {module};"
        );
        let _ = writeln!(out, "    pub use {module}::*;");
    }
    out.push_str("}\n");

    fs::write(out_dir.join("framework_generated.rs"), out)
        .expect("failed to write framework generated root include");
}
