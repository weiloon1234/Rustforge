use std::sync::Arc;

use async_trait::async_trait;
use axum::http::HeaderMap;
use core_datatable::{
    DataTableActor, DataTableAsyncExportManager, DataTableContext, DataTableRegistry,
};
use core_db::infra::storage::Storage;
use core_web::auth::Guard;
use core_web::datatable::{
    DataTableEmailExportManager, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use serde_json::Value;

use generated::guards::AdminGuard;

use crate::contracts::datatable::admin::account::AdminAdminDataTableContract;
use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    core_web::datatable::routes_for_scoped_contract_with_options(
        "/datatable/admin",
        state,
        AdminAdminDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}

#[async_trait]
impl DataTableRouteState for AppApiState {
    fn datatable_registry(&self) -> &Arc<DataTableRegistry> {
        &self.datatable_registry
    }

    fn datatable_async_exports(&self) -> &Arc<DataTableAsyncExportManager> {
        &self.datatable_async_exports
    }

    fn datatable_storage(&self) -> &Arc<dyn Storage> {
        &self.storage
    }

    fn datatable_mailer(&self) -> &Arc<core_mailer::Mailer> {
        &self.mailer
    }

    fn datatable_email_exports(&self) -> &Arc<DataTableEmailExportManager> {
        &self.datatable_email_exports
    }

    fn datatable_export_link_ttl_secs(&self) -> u64 {
        self.datatable_export_link_ttl_secs
    }

    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext {
        let actor = build_admin_actor(&self.db, headers).await;
        DataTableContext {
            default_per_page: self.datatable_default_per_page,
            app_timezone: self.app_timezone.clone(),
            user_timezone: core_web::utils::datatable::parse_timezone_from_headers(headers),
            actor,
            unknown_filter_mode: self.datatable_unknown_filter_mode,
        }
    }
}

async fn build_admin_actor(db: &sqlx::PgPool, headers: &HeaderMap) -> Option<DataTableActor> {
    let token = core_web::auth::extract_bearer_token(headers)?;
    let auth = core_web::auth::authenticate_token::<AdminGuard>(db, &token)
        .await
        .ok()?;

    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        "admin_type".to_string(),
        Value::String(auth.user.admin_type.as_str().to_string()),
    );

    Some(DataTableActor {
        id: auth.subject_id.clone(),
        guard: Some(AdminGuard::name().to_string()),
        roles: Vec::new(),
        permissions: auth.abilities,
        attributes,
    })
}
