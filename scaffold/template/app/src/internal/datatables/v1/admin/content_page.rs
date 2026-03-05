use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{ContentPageDataTable, ContentPageDataTableHooks},
    permissions::Permission,
};

use crate::contracts::datatable::admin::content_page::{
    AdminContentPageDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct ContentPageDataTableAppHooks;

impl ContentPageDataTableHooks for ContentPageDataTableAppHooks {
    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::ContentPageRead.as_str(),
                Permission::ContentPageManage.as_str(),
            ],
            PermissionMode::Any,
        );

        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn row_to_record(
        &self,
        row: generated::models::ContentPageView,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row)?;
        record.remove("title_translations");
        record.remove("content_translations");
        record.remove("cover_translations");
        record.remove("created_at");
        record.remove("deleted_at");
        Ok(record)
    }
}

pub type AppContentPageDataTable = ContentPageDataTable<ContentPageDataTableAppHooks>;

pub fn app_content_page_datatable(db: sqlx::PgPool) -> AppContentPageDataTable {
    ContentPageDataTable::new(db).with_hooks(ContentPageDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_content_page_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminContentPageDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
