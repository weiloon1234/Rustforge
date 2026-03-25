pub mod config;
pub mod gen_auth;
pub mod gen_datatables;
pub mod gen_enums;
pub mod gen_localized;
pub mod gen_models;
pub mod gen_permissions;
pub mod permissions;
pub mod schema;
pub mod template;

pub use config::ConfigsFile;
pub use gen_auth::generate_auth;
pub use gen_datatables::generate_datatable_skeletons;
pub use gen_enums::{
    generate_enum_with_options, generate_enums, generate_enums_with_options, GenerateEnumsOptions,
};
pub use gen_localized::generate_localized;
pub use gen_models::{generate_models, generate_models_with_options, GenerateModelsOptions};
pub use gen_permissions::generate_permissions;
pub use permissions::load_permissions;
/// Escape a path for use inside a `#[path = "..."]` attribute.
/// Replaces backslashes with double-backslashes for Windows compatibility.
pub fn escape_path_for_include(p: &std::path::Path) -> String {
    p.display().to_string().replace('\\', "\\\\").replace('"', "\\\"")
}

/// Write file only when content differs — prevents spurious `git status` noise
/// from code generation that produces identical output.
pub fn write_if_changed(path: &std::path::Path, content: impl AsRef<[u8]>) -> std::io::Result<()> {
    let content = content.as_ref();
    if path.exists() {
        if let Ok(existing) = std::fs::read(path) {
            if existing == content {
                return Ok(());
            }
        }
    }
    std::fs::write(path, content)
}

pub use schema::{
    framework_model_source_paths_from_core_db, load_framework, load_framework_from_paths,
    load_framework_from_sources, load_with_framework, load_with_framework_from_paths,
    load_with_framework_from_sources, FrameworkModelSource, Schema,
};
