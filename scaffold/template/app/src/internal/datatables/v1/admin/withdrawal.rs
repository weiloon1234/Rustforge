use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

use crate::contracts::datatable::admin::withdrawal::{
    AdminWithdrawalDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct WithdrawalDataTableAppHooks;

impl WithdrawalDataTableHooks for WithdrawalDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, WithdrawalModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, WithdrawalModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::WithdrawalRead.as_str(), Permission::WithdrawalManage.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, WithdrawalModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, WithdrawalModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-owner_type" => {
                if let Some(ot) = OwnerType::from_storage(value) {
                    Ok(Some(query.where_col(WithdrawalCol::OWNER_TYPE, Op::Eq, ot)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-credit_type" => {
                if let Some(ct) = CreditType::from_storage(value) {
                    Ok(Some(query.where_col(WithdrawalCol::CREDIT_TYPE, Op::Eq, ct)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-withdrawal_method" => {
                if let Some(wm) = WithdrawalMethod::from_storage(value) {
                    Ok(Some(query.where_col(WithdrawalCol::WITHDRAWAL_METHOD, Op::Eq, wm)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-status" => {
                if let Some(s) = WithdrawalStatus::from_storage(value) {
                    Ok(Some(query.where_col(WithdrawalCol::STATUS, Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut WithdrawalRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: WithdrawalRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert(
            "status_label".into(),
            serde_json::Value::String(row.status_label()),
        );
        record.insert(
            "admin_username".into(),
            row.admin
                .as_ref()
                .map(|a| serde_json::Value::String(a.username.clone()))
                .unwrap_or(serde_json::Value::Null),
        );
        record.insert("owner_name".into(), serde_json::Value::Null);
        record.insert(
            "bank_name".into(),
            row.bank
                .as_ref()
                .map(|b| serde_json::Value::String(b.name.clone()))
                .unwrap_or(serde_json::Value::Null),
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
    query: Query<'db, WithdrawalModel>,
    value: &str,
) -> Query<'db, WithdrawalModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_col(WithdrawalCol::ID, Op::Eq, id);
    }
    let pattern = format!("%{trimmed}%");
    query.where_col(WithdrawalCol::RELATED_KEY, Op::Like, Some(pattern))
}

pub type AppWithdrawalDataTable = WithdrawalDataTable<WithdrawalDataTableAppHooks>;

pub fn app_withdrawal_datatable(db: sqlx::PgPool) -> AppWithdrawalDataTable {
    WithdrawalDataTable::new(db).with_hooks(WithdrawalDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_withdrawal_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminWithdrawalDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
