// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from app/schemas to bootstrap app-level datatable hooks.

pub mod admin;
pub use admin::{AdminDataTableAppHooks, app_admin_datatable, app_admin_datatable_with_config, register_admin_datatable};

use core_datatable::DataTableRegistry;

#[allow(unused_variables)]
pub fn register_all_generated_datatables(registry: &mut DataTableRegistry, db: &sqlx::PgPool) {
    register_admin_datatable(registry, db.clone());
}
