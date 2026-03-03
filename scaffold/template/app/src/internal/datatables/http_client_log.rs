// Compatibility module for db-gen's `pub mod http_client_log;` output.
// Custom v1-scoped datatable hooks live under `v1/admin/http_client_log.rs`.
pub use crate::internal::datatables::v1::admin::{
    app_http_client_log_datatable, app_http_client_log_datatable_with_config,
    register_http_client_log_datatable, AppHttpClientLogDataTable, HttpClientLogDataTableAppHooks,
};
