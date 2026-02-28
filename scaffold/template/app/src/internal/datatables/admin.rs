// Compatibility module for db-gen's `pub mod admin;` output.
// Custom portal-scoped datatable hooks live under `portal/admin/account.rs`.
pub use crate::internal::datatables::portal::admin::{
    app_admin_datatable, app_admin_datatable_with_config, register_admin_datatable,
    AdminDataTableAppHooks, AppAdminDataTable,
};
