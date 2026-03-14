use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{
        AdminCol, IntroducerChangeDataTable, IntroducerChangeDataTableHooks,
        IntroducerChangeModel, IntroducerChangeRecord, IntroducerChangeRel, UserCol,
    },
    permissions::Permission,
};

use crate::contracts::datatable::admin::introducer_change::{
    AdminIntroducerChangeDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct IntroducerChangeDataTableAppHooks;

impl IntroducerChangeDataTableHooks for IntroducerChangeDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, IntroducerChangeModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, IntroducerChangeModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::UserChangeIntroducer.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, IntroducerChangeModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, IntroducerChangeModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            _ => Ok(None),
        }
    }

    fn row_to_record(
        &self,
        row: IntroducerChangeRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert("user_username".into(),
            row.user.as_ref().map(|u| serde_json::Value::String(u.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        record.insert("from_username".into(),
            row.from_user.as_ref().map(|u| serde_json::Value::String(u.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        record.insert("to_username".into(),
            row.to_user.as_ref().map(|u| serde_json::Value::String(u.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        record.insert("admin_username".into(),
            row.admin.as_ref().map(|a| serde_json::Value::String(a.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(
    query: Query<'db, IntroducerChangeModel>,
    value: &str,
) -> Query<'db, IntroducerChangeModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    query
        .where_has(IntroducerChangeRel::USER, |rq| rq.where_col(UserCol::USERNAME, Op::Like, pattern.clone()))
        .or_where_has(IntroducerChangeRel::FROM_USER, |rq| rq.where_col(UserCol::USERNAME, Op::Like, pattern.clone()))
        .or_where_has(IntroducerChangeRel::TO_USER, |rq| rq.where_col(UserCol::USERNAME, Op::Like, pattern.clone()))
        .or_where_has(IntroducerChangeRel::ADMIN, |rq| rq.where_col(AdminCol::USERNAME, Op::Like, pattern))
}

pub type AppIntroducerChangeDataTable =
    IntroducerChangeDataTable<IntroducerChangeDataTableAppHooks>;

pub fn app_introducer_change_datatable(
    db: sqlx::PgPool,
) -> AppIntroducerChangeDataTable {
    IntroducerChangeDataTable::new(db).with_hooks(IntroducerChangeDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_introducer_change_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminIntroducerChangeDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
