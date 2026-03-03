// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from app/schemas to bootstrap app-level datatable hooks.

pub mod admin;
pub use admin::{AdminDataTableAppHooks, app_admin_datatable, app_admin_datatable_with_config, register_admin_datatable};
pub mod http_client_log;
pub use http_client_log::{HttpClientLogDataTableAppHooks, app_http_client_log_datatable, app_http_client_log_datatable_with_config, register_http_client_log_datatable};
pub mod page;
pub use page::{PageDataTableAppHooks, app_page_datatable, app_page_datatable_with_config, register_page_datatable};
pub mod webhook_log;
pub use webhook_log::{WebhookLogDataTableAppHooks, app_webhook_log_datatable, app_webhook_log_datatable_with_config, register_webhook_log_datatable};

use core_datatable::DataTableRegistry;

#[allow(unused_variables)]
pub fn register_all_generated_datatables(registry: &mut DataTableRegistry, db: &sqlx::PgPool) {
    register_admin_datatable(registry, db.clone());
    register_http_client_log_datatable(registry, db.clone());
    register_page_datatable(registry, db.clone());
    register_webhook_log_datatable(registry, db.clone());
}
