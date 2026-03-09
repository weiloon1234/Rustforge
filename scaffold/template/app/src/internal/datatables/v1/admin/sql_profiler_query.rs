use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::models::{SqlProfilerQueryDataTable, SqlProfilerQueryDataTableHooks};

use crate::contracts::datatable::admin::sql_profiler_query::{
    AdminSqlProfilerQueryDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct SqlProfilerQueryDataTableAppHooks;

impl SqlProfilerQueryDataTableHooks for SqlProfilerQueryDataTableAppHooks {
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

pub type AppSqlProfilerQueryDataTable = SqlProfilerQueryDataTable<SqlProfilerQueryDataTableAppHooks>;

pub fn app_sql_profiler_query_datatable(db: sqlx::PgPool) -> AppSqlProfilerQueryDataTable {
    SqlProfilerQueryDataTable::new(db).with_hooks(SqlProfilerQueryDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_sql_profiler_query_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminSqlProfilerQueryDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
