// Compatibility module for db-gen's `pub mod page;` output.
// Custom v1-scoped datatable hooks live under `v1/admin/page.rs`.
pub use crate::internal::datatables::v1::admin::{
    app_page_datatable, app_page_datatable_with_config, register_page_datatable,
    AppPageDataTable, PageDataTableAppHooks,
};
