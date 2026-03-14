use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

use crate::contracts::datatable::admin::company_bank_account::{
    AdminCompanyBankAccountDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct CompanyBankAccountDataTableAppHooks;

impl CompanyBankAccountDataTableHooks for CompanyBankAccountDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, CompanyBankAccountModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, CompanyBankAccountModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::CompanyBankAccountRead.as_str(),
                Permission::CompanyBankAccountManage.as_str(),
            ],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, CompanyBankAccountModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, CompanyBankAccountModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-bank_id" => {
                if let Ok(bank_id) = value.trim().parse::<i64>() {
                    Ok(Some(query.where_col(CompanyBankAccountCol::BANK_ID, Op::Eq, bank_id)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-status" => {
                if let Some(s) = CompanyBankAccountStatus::from_storage(value) {
                    Ok(Some(query.where_col(CompanyBankAccountCol::STATUS, Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut CompanyBankAccountRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: CompanyBankAccountRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert(
            "status_label".into(),
            serde_json::Value::String(row.status.explained_label().to_string()),
        );
        record.insert(
            "bank_name".into(),
            row.bank
                .as_ref()
                .map(|b| serde_json::Value::String(b.name.clone()))
                .unwrap_or(serde_json::Value::Null),
        );
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(
    query: Query<'db, CompanyBankAccountModel>,
    value: &str,
) -> Query<'db, CompanyBankAccountModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_col(CompanyBankAccountCol::ID, Op::Eq, id);
    }
    let pattern = format!("%{trimmed}%");
    query.where_col(CompanyBankAccountCol::ACCOUNT_NAME, Op::Like, pattern)
}

pub type AppCompanyBankAccountDataTable =
    CompanyBankAccountDataTable<CompanyBankAccountDataTableAppHooks>;

pub fn app_company_bank_account_datatable(db: sqlx::PgPool) -> AppCompanyBankAccountDataTable {
    CompanyBankAccountDataTable::new(db)
        .with_hooks(CompanyBankAccountDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_company_bank_account_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminCompanyBankAccountDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
