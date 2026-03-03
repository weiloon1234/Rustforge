use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use axum::http::HeaderMap;
use core_datatable::{
    DataTableActor, DataTableAsyncExportManager, DataTableContext, DataTableRegistry,
};
use core_db::infra::storage::Storage;
use core_i18n::t;
use core_web::auth::Guard;
use core_web::datatable::{
    DataTableEmailExportManager, DataTableGenericQueryRequest, DataTableQueryRequestContract,
    DataTableRouteState,
};
use core_web::error::AppError;
use core_web::openapi::{aide::axum::routing::post_with, ApiRouter};
use core_web::{contracts::ContractJson, response::ApiResponse};
use serde_json::Value;

use generated::guards::AdminGuard;
use generated::models::AdminDataTableHooks;

use crate::contracts::datatable::admin::account::{
    AdminDatatableSummaryOutput, SCOPED_KEY as ADMIN_ACCOUNT_SCOPED_KEY,
};
use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    let scoped_datatable_routes =
        crate::internal::datatables::v1::admin::mount_scoped_datatable_routes(state.clone());
    let summary_route = ApiRouter::new()
        .api_route(
            "/datatable/admin/summary",
            post_with(admin_summary, |op| {
                op.summary("Datatable summary")
                    .tag("Admin Account")
                    .description("Returns filtered cross-page summary cards for admin datatable.")
            }),
        )
        .with_state(state);

    scoped_datatable_routes.merge(summary_route)
}

async fn admin_summary(
    State(state): State<AppApiState>,
    headers: core_web::extract::request_headers::RequestHeaders,
    req: ContractJson<DataTableGenericQueryRequest>,
) -> Result<ApiResponse<AdminDatatableSummaryOutput>, AppError> {
    let mut input = req.0.datatable_query_to_input();
    input.model = Some(ADMIN_ACCOUNT_SCOPED_KEY.to_string());

    let ctx = state.datatable_context(&headers).await;
    let hooks = crate::internal::datatables::v1::admin::AdminDataTableAppHooks::default();
    if !hooks.authorize(&input, &ctx)? {
        return Err(AppError::Forbidden(t(
            "You are not allowed to query this datatable",
        )));
    }

    let summary =
        crate::internal::datatables::v1::admin::build_admin_summary_output(&state.db, &input, &ctx)
            .await?;

    Ok(ApiResponse::success(summary, &t("datatable summary")))
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
