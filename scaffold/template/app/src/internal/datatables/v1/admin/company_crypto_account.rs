use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

use crate::contracts::datatable::admin::company_crypto_account::{
    AdminCompanyCryptoAccountDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct CompanyCryptoAccountDataTableAppHooks;

impl CompanyCryptoAccountDataTableHooks for CompanyCryptoAccountDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, CompanyCryptoAccountModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, CompanyCryptoAccountModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::CompanyCryptoAccountRead.as_str(),
                Permission::CompanyCryptoAccountManage.as_str(),
            ],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, CompanyCryptoAccountModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, CompanyCryptoAccountModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-crypto_network_id" => {
                if let Ok(nid) = value.trim().parse::<i64>() {
                    Ok(Some(query.where_col(CompanyCryptoAccountCol::CRYPTO_NETWORK_ID, Op::Eq, nid)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-status" => {
                if let Some(s) = CompanyCryptoAccountStatus::from_storage(value) {
                    Ok(Some(query.where_col(CompanyCryptoAccountCol::STATUS, Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut CompanyCryptoAccountRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: CompanyCryptoAccountRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert(
            "status_label".into(),
            serde_json::Value::String(row.status.explained_label().to_string()),
        );
        record.insert(
            "crypto_network_name".into(),
            row.crypto_network
                .as_ref()
                .map(|n| serde_json::Value::String(n.name.clone()))
                .unwrap_or(serde_json::Value::Null),
        );
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(
    query: Query<'db, CompanyCryptoAccountModel>,
    value: &str,
) -> Query<'db, CompanyCryptoAccountModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_col(CompanyCryptoAccountCol::ID, Op::Eq, id);
    }
    let pattern = format!("%{trimmed}%");
    query.where_col(CompanyCryptoAccountCol::WALLET_ADDRESS, Op::Like, pattern)
}

pub type AppCompanyCryptoAccountDataTable =
    CompanyCryptoAccountDataTable<CompanyCryptoAccountDataTableAppHooks>;

pub fn app_company_crypto_account_datatable(
    db: sqlx::PgPool,
) -> AppCompanyCryptoAccountDataTable {
    CompanyCryptoAccountDataTable::new(db)
        .with_hooks(CompanyCryptoAccountDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_company_crypto_account_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminCompanyCryptoAccountDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
