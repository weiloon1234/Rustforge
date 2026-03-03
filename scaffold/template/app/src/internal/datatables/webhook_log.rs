// Compatibility module for db-gen's `pub mod webhook_log;` output.
// Custom v1-scoped datatable hooks live under `v1/admin/webhook_log.rs`.
pub use crate::internal::datatables::v1::admin::{
    app_webhook_log_datatable, app_webhook_log_datatable_with_config, register_webhook_log_datatable,
    AppWebhookLogDataTable, WebhookLogDataTableAppHooks,
};
