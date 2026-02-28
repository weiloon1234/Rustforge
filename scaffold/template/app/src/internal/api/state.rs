use std::sync::Arc;

use bootstrap::boot::BootContext;
use core_config::DataTableUnknownFilterMode as ConfigUnknownFilterMode;
use core_datatable::{DataTableAsyncExportManager, DataTableRegistry, DataTableUnknownFilterMode};
use core_db::infra::storage::Storage;
use core_web::datatable::DataTableEmailExportManager;

#[derive(Clone)]
pub struct AppApiState {
    pub db: sqlx::PgPool,
    pub auth: core_config::AuthSettings,
    pub storage: Arc<dyn Storage>,
    pub mailer: Arc<core_mailer::Mailer>,
    pub datatable_registry: Arc<DataTableRegistry>,
    pub datatable_async_exports: Arc<DataTableAsyncExportManager>,
    pub datatable_email_exports: Arc<DataTableEmailExportManager>,
    pub datatable_default_per_page: i64,
    pub datatable_unknown_filter_mode: DataTableUnknownFilterMode,
    pub datatable_export_link_ttl_secs: u64,
    pub app_timezone: String,
}

impl AppApiState {
    pub fn new(ctx: &BootContext) -> anyhow::Result<Self> {
        let mut datatable_registry = DataTableRegistry::new();
        crate::internal::datatables::register_all_generated_datatables(&mut datatable_registry, &ctx.db);
        datatable_registry.register_as(
            "admin.account",
            crate::internal::datatables::app_admin_datatable(ctx.db.clone()),
        );

        let datatable_registry = Arc::new(datatable_registry);
        let datatable_async_exports =
            Arc::new(DataTableAsyncExportManager::new(datatable_registry.clone()));

        Ok(Self {
            db: ctx.db.clone(),
            auth: ctx.settings.auth.clone(),
            storage: ctx.storage.clone(),
            mailer: ctx.mailer.clone(),
            datatable_registry,
            datatable_async_exports,
            datatable_email_exports: Arc::new(DataTableEmailExportManager::new()),
            datatable_default_per_page: ctx.settings.app.default_per_page as i64,
            datatable_unknown_filter_mode: map_unknown_filter_mode(
                ctx.settings.app.datatable_unknown_filter_mode,
            ),
            datatable_export_link_ttl_secs: ctx.settings.app.datatable_export_link_ttl_secs,
            app_timezone: ctx.settings.i18n.default_timezone_str.clone(),
        })
    }
}

impl core_web::auth::AuthState for AppApiState {
    fn auth_db(&self) -> &sqlx::PgPool {
        &self.db
    }
}

impl core_web::extract::GetDb for AppApiState {
    fn db(&self) -> &sqlx::PgPool {
        &self.db
    }
}

fn map_unknown_filter_mode(mode: ConfigUnknownFilterMode) -> DataTableUnknownFilterMode {
    match mode {
        ConfigUnknownFilterMode::Ignore => DataTableUnknownFilterMode::Ignore,
        ConfigUnknownFilterMode::Warn => DataTableUnknownFilterMode::Warn,
        ConfigUnknownFilterMode::Error => DataTableUnknownFilterMode::Error,
    }
}
