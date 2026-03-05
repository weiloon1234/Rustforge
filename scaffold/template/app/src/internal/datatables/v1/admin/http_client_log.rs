use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::models::{HttpClientLogDataTable, HttpClientLogDataTableHooks};

use crate::contracts::datatable::admin::http_client_log::{
    AdminHttpClientLogDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct HttpClientLogDataTableAppHooks;

impl HttpClientLogDataTableHooks for HttpClientLogDataTableAppHooks {
    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        Ok(authorize_with_optional_export(
            is_developer_actor(ctx),
            input,
            ctx,
        ))
    }
}

fn is_developer_actor(ctx: &DataTableContext) -> bool {
    ctx.actor
        .as_ref()
        .and_then(|actor| actor.attributes.get("admin_type"))
        .and_then(|value| value.as_str())
        .is_some_and(|admin_type| admin_type.eq_ignore_ascii_case("developer"))
}

pub type AppHttpClientLogDataTable = HttpClientLogDataTable<HttpClientLogDataTableAppHooks>;

pub fn app_http_client_log_datatable(db: sqlx::PgPool) -> AppHttpClientLogDataTable {
    HttpClientLogDataTable::new(db).with_hooks(HttpClientLogDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_http_client_log_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminHttpClientLogDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
