use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::models::{SqlProfilerRequestDataTable, SqlProfilerRequestDataTableConfig, SqlProfilerRequestDataTableHooks};

use crate::contracts::datatable::admin::sql_profiler_request::{
    AdminSqlProfilerRequestDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct SqlProfilerRequestDataTableAppHooks;

impl SqlProfilerRequestDataTableHooks for SqlProfilerRequestDataTableAppHooks {
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

pub type AppSqlProfilerRequestDataTable = SqlProfilerRequestDataTable<SqlProfilerRequestDataTableAppHooks>;

pub fn app_sql_profiler_request_datatable(db: sqlx::PgPool) -> AppSqlProfilerRequestDataTable {
    SqlProfilerRequestDataTable::new(db)
        .with_hooks(SqlProfilerRequestDataTableAppHooks::default())
        .with_config(SqlProfilerRequestDataTableConfig {
            default_sorting_column: "created_at",
            ..SqlProfilerRequestDataTableConfig::default()
        })
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_sql_profiler_request_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminSqlProfilerRequestDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
