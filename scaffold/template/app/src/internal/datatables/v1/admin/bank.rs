use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

use crate::contracts::datatable::admin::bank::{
    AdminBankDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct BankDataTableAppHooks;

impl BankDataTableHooks for BankDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, BankModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, BankModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::BankRead.as_str(), Permission::BankManage.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, BankModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, BankModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-country_iso2" => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Ok(Some(query))
                } else {
                    Ok(Some(query.where_col(BankCol::COUNTRY_ISO2, Op::Eq, trimmed.to_string())))
                }
            }
            "f-status" => {
                if let Some(s) = BankStatus::from_storage(value) {
                    Ok(Some(query.where_col(BankCol::STATUS, Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut BankRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: BankRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert(
            "status_label".into(),
            serde_json::Value::String(row.status.explained_label().to_string()),
        );
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(query: Query<'db, BankModel>, value: &str) -> Query<'db, BankModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_col(BankCol::ID, Op::Eq, id);
    }
    let pattern = format!("%{trimmed}%");
    query.where_col(BankCol::NAME, Op::Like, pattern)
}

pub type AppBankDataTable = BankDataTable<BankDataTableAppHooks>;

pub fn app_bank_datatable(db: sqlx::PgPool) -> AppBankDataTable {
    BankDataTable::new(db).with_hooks(BankDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_bank_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminBankDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
