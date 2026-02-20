use crate::schema::{to_snake, to_title_case, Schema};
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::Write as IoWrite;
use std::path::Path;

pub fn generate_datatable_skeletons(schema: &Schema, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;

    let mod_path = out_dir.join("mod.rs");
    if !mod_path.exists() {
        fs::write(&mod_path, "include!(\"mod.generated.rs\");\n")?;
    }

    let mut generated = String::new();
    generated.push_str("// AUTO-GENERATED FILE — DO NOT EDIT\n");
    generated.push_str("// Generated from app/schemas to bootstrap app-level datatable hooks.\n\n");

    for model_name in schema.models.keys() {
        let model_snake = to_snake(model_name);
        let model_title = to_title_case(&model_snake);
        let file_path = out_dir.join(format!("{model_snake}.rs"));

        writeln!(generated, "pub mod {model_snake};")?;
        writeln!(
            generated,
            "pub use {model_snake}::{{{model_title}DataTableAppHooks, app_{model_snake}_datatable, app_{model_snake}_datatable_with_config, register_{model_snake}_datatable}};"
        )?;

        if !file_path.exists() {
            let mut f = fs::File::create(&file_path)?;
            writeln!(
                f,
                "// App-level datatable hooks for {model_title}.\n// Generated once by db-gen; safe to edit.\n"
            )?;
            writeln!(f, "use core_datatable::DataTableRegistry;")?;
            writeln!(
                f,
                "use generated::models::{{{model_title}DataTable, {model_title}DataTableConfig, {model_title}DataTableHooks}};"
            )?;
            writeln!(f)?;
            writeln!(f, "#[derive(Default, Clone)]")?;
            writeln!(f, "pub struct {model_title}DataTableAppHooks;")?;
            writeln!(f)?;
            writeln!(
                f,
                "impl {model_title}DataTableHooks for {model_title}DataTableAppHooks {{"
            )?;
            writeln!(
                f,
                "    // Override scope/authorize/filters/mappings when needed."
            )?;
            writeln!(f, "}}\n")?;
            writeln!(
                f,
                "pub type App{model_title}DataTable = {model_title}DataTable<{model_title}DataTableAppHooks>;\n"
            )?;
            writeln!(
                f,
                "pub fn app_{model_snake}_datatable(db: sqlx::PgPool) -> App{model_title}DataTable {{"
            )?;
            writeln!(
                f,
                "    {model_title}DataTable::new(db).with_hooks({model_title}DataTableAppHooks::default())"
            )?;
            writeln!(f, "}}\n")?;
            writeln!(f, "pub fn app_{model_snake}_datatable_with_config(")?;
            writeln!(f, "    db: sqlx::PgPool,")?;
            writeln!(f, "    config: {model_title}DataTableConfig,")?;
            writeln!(f, ") -> App{model_title}DataTable {{")?;
            writeln!(
                f,
                "    {model_title}DataTable::new(db).with_hooks({model_title}DataTableAppHooks::default()).with_config(config)"
            )?;
            writeln!(f, "}}\n")?;
            writeln!(
                f,
                "pub fn register_{model_snake}_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {{"
            )?;
            writeln!(f, "    registry.register(app_{model_snake}_datatable(db));")?;
            writeln!(f, "}}")?;
        }
    }

    writeln!(
        generated,
        "\nuse core_datatable::DataTableRegistry;\n\n#[allow(unused_variables)]\npub fn register_all_generated_datatables(registry: &mut DataTableRegistry, db: &sqlx::PgPool) {{"
    )?;
    for model_name in schema.models.keys() {
        let model_snake = to_snake(model_name);
        writeln!(
            generated,
            "    register_{model_snake}_datatable(registry, db.clone());"
        )?;
    }
    writeln!(generated, "}}")?;

    fs::write(out_dir.join("mod.generated.rs"), generated)?;
    Ok(())
}
