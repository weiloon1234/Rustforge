// Compatibility module for db-gen's `pub mod admin;` output.
// Custom v1-scoped datatable hooks live under `v1/admin/account.rs`.
pub use crate::internal::datatables::v1::admin::{
    app_admin_datatable, app_admin_datatable_with_config, register_admin_datatable,
    AdminDataTableAppHooks, AppAdminDataTable,
};
