// App-level datatable hooks for {{model_title}}.
// Generated once by db-gen; safe to edit.

use core_datatable::DataTableRegistry;
use generated::models::{ {{model_title}}DataTable, {{model_title}}DataTableConfig, {{model_title}}DataTableHooks };

#[derive(Default, Clone)]
pub struct {{model_title}}DataTableAppHooks;

impl {{model_title}}DataTableHooks for {{model_title}}DataTableAppHooks {
    // Override scope/authorize/filters/mappings/summary when needed.
}

pub type App{{model_title}}DataTable = {{model_title}}DataTable<{{model_title}}DataTableAppHooks>;

pub fn app_{{model_snake}}_datatable(db: sqlx::PgPool) -> App{{model_title}}DataTable {
    {{model_title}}DataTable::new(db).with_hooks({{model_title}}DataTableAppHooks::default())
}

pub fn app_{{model_snake}}_datatable_with_config(
    db: sqlx::PgPool,
    config: {{model_title}}DataTableConfig,
) -> App{{model_title}}DataTable {
    {{model_title}}DataTable::new(db)
        .with_hooks({{model_title}}DataTableAppHooks::default())
        .with_config(config)
}

pub fn register_{{model_snake}}_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register(app_{{model_snake}}_datatable(db));
}
