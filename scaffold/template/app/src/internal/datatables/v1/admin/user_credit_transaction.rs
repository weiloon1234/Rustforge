use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{models::*, permissions::Permission};

use crate::contracts::datatable::admin::user_credit_transaction::{
    AdminUserCreditTransactionDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct UserCreditTransactionDataTableAppHooks;

impl UserCreditTransactionDataTableHooks for UserCreditTransactionDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, UserCreditTransactionModel>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Query<'db, UserCreditTransactionModel> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::UserCreditRead.as_str(), Permission::UserCreditManage.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, UserCreditTransactionModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, UserCreditTransactionModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-credit_type" => {
                if let Some(ct) = CreditType::from_storage(value) {
                    Ok(Some(query.where_col(UserCreditTransactionCol::CREDIT_TYPE, Op::Eq, ct)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-transaction_type" => {
                if let Some(tt) = CreditTransactionType::from_storage(value) {
                    Ok(Some(query.where_col(UserCreditTransactionCol::TRANSACTION_TYPE, Op::Eq, tt)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-user_id" => {
                if let Ok(uid) = value.trim().parse::<i64>() {
                    Ok(Some(query.where_col(UserCreditTransactionCol::USER_ID, Op::Eq, uid)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        row: &mut generated::models::UserCreditTransactionRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        row.enrich_transaction_type_explained();
        Ok(())
    }

    fn row_to_record(
        &self,
        row: generated::models::UserCreditTransactionRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row.clone())?;
        record.insert("user_username".into(),
            row.user.as_ref().map(|u| serde_json::Value::String(u.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        record.insert("admin_username".into(),
            row.admin.as_ref().map(|a| serde_json::Value::String(a.username.clone()))
                .unwrap_or(serde_json::Value::Null));
        Ok(record)
    }
}

fn apply_keyword_filter<'db>(
    query: Query<'db, UserCreditTransactionModel>,
    value: &str,
) -> Query<'db, UserCreditTransactionModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    query.where_has(UserCreditTransactionRel::USER, |rq| rq.where_col(UserCol::USERNAME, Op::Like, pattern))
}

pub type AppUserCreditTransactionDataTable =
    UserCreditTransactionDataTable<UserCreditTransactionDataTableAppHooks>;

pub fn app_user_credit_transaction_datatable(
    db: sqlx::PgPool,
) -> AppUserCreditTransactionDataTable {
    UserCreditTransactionDataTable::new(db)
        .with_hooks(UserCreditTransactionDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_user_credit_transaction_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminUserCreditTransactionDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
