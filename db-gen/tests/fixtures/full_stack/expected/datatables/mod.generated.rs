// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from app/schemas to bootstrap app-level datatable hooks.

pub mod admin;
pub use admin::{AdminDataTableAppHooks, app_admin_datatable, app_admin_datatable_with_config, register_admin_datatable};
pub mod article;
pub use article::{ArticleDataTableAppHooks, app_article_datatable, app_article_datatable_with_config, register_article_datatable};
pub mod profile;
pub use profile::{ProfileDataTableAppHooks, app_profile_datatable, app_profile_datatable_with_config, register_profile_datatable};
pub mod user;
pub use user::{UserDataTableAppHooks, app_user_datatable, app_user_datatable_with_config, register_user_datatable};

use core_datatable::DataTableRegistry;

#[allow(unused_variables)]
pub fn register_all_generated_datatables(registry: &mut DataTableRegistry, db: &sqlx::PgPool) {
    register_admin_datatable(registry, db.clone());
    register_article_datatable(registry, db.clone());
    register_profile_datatable(registry, db.clone());
    register_user_datatable(registry, db.clone());
}
