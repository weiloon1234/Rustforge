use crate::schema::{to_snake, to_title_case, Schema};
use crate::template::{render_template, TemplateContext};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct DatatableModelSpec {
    snake: String,
    title: String,
}

pub fn generate_datatable_skeletons(schema: &Schema, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;
    let models = collect_datatable_models(schema);

    let mod_path = out_dir.join("mod.rs");
    if !mod_path.exists() {
        fs::write(&mod_path, "include!(\"mod.generated.rs\");\n")?;
    }

    let module_exports = render_module_exports(&models)?;
    let register_all_calls = render_register_all_calls(&models)?;
    let mut mod_context = TemplateContext::new();
    mod_context.insert("module_exports", module_exports)?;
    mod_context.insert("register_all_calls", register_all_calls)?;
    let generated = render_template("datatables/mod.generated.rs.tpl", &mod_context)?;
    fs::write(out_dir.join("mod.generated.rs"), generated)?;

    for model in &models {
        let file_path = out_dir.join(format!("{}.rs", model.snake));
        if file_path.exists() {
            continue;
        }

        let mut model_context = TemplateContext::new();
        model_context.insert("model_title", model.title.clone())?;
        model_context.insert("model_snake", model.snake.clone())?;
        let rendered = render_template("datatables/model.rs.tpl", &model_context)?;
        fs::write(file_path, rendered)?;
    }

    Ok(())
}

fn collect_datatable_models(schema: &Schema) -> Vec<DatatableModelSpec> {
    schema
        .models
        .keys()
        .map(|model_name| {
            let snake = to_snake(model_name);
            let title = to_title_case(&snake);
            DatatableModelSpec { snake, title }
        })
        .collect()
}

fn render_module_exports(models: &[DatatableModelSpec]) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for model in models {
        writeln!(out, "{}", render_module_export_entry(model))?;
    }
    Ok(out.trim_end().to_string())
}

fn render_module_export_entry(model: &DatatableModelSpec) -> String {
    format!(
        "pub mod {snake};\npub use {snake}::{{{title}DataTableAppHooks, app_{snake}_datatable, app_{snake}_datatable_with_config, register_{snake}_datatable}};",
        snake = model.snake,
        title = model.title,
    )
}

fn render_register_all_calls(models: &[DatatableModelSpec]) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for model in models {
        writeln!(out, "{}", render_register_call(model))?;
    }
    Ok(out.trim_end().to_string())
}

fn render_register_call(model: &DatatableModelSpec) -> String {
    format!(
        "    register_{}_datatable(registry, db.clone());",
        model.snake
    )
}
