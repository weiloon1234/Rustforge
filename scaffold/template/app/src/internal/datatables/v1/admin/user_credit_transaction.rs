use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::{Op, RawClause};
use core_i18n::t_args;
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{
        CreditTransactionType, CreditType, UserCreditTransactionDataTable,
        UserCreditTransactionDataTableHooks, UserCreditTransactionQuery,
        UserCreditTransactionView,
    },
    permissions::Permission,
};

use crate::contracts::datatable::admin::user_credit_transaction::{
    AdminUserCreditTransactionDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct UserCreditTransactionDataTableAppHooks;

impl UserCreditTransactionDataTableHooks for UserCreditTransactionDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: UserCreditTransactionQuery<'db>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> UserCreditTransactionQuery<'db> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::UserCredit.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: UserCreditTransactionQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<UserCreditTransactionQuery<'db>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-credit_type" => {
                if let Some(ct) = CreditType::from_storage(value) {
                    Ok(Some(query.where_credit_type(Op::Eq, ct)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-transaction_type" => {
                if let Some(tt) = CreditTransactionType::from_storage(value) {
                    Ok(Some(query.where_transaction_type(Op::Eq, tt)))
                } else {
                    Ok(Some(query))
                }
            }
            "f-user_id" => {
                if let Ok(uid) = value.trim().parse::<i64>() {
                    Ok(Some(query.where_user_id(Op::Eq, uid)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        row: &mut generated::models::UserCreditTransactionWithRelations,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        enrich_transaction_type_explained(&mut *row);
        Ok(())
    }
}

/// Enrich the generated `transaction_type_explained` on the View directly.
///
/// Priority:
/// - `remark` non-empty → use remark as the explanation
/// - `params` non-empty → re-translate with `t_args` to interpolate `:param` placeholders
/// - otherwise → keep the generated `explained_label()` as-is
fn enrich_transaction_type_explained(row: &mut UserCreditTransactionView) {
    if let Some(ref remark) = row.remark {
        let trimmed = remark.trim();
        if !trimmed.is_empty() {
            row.transaction_type_explained = trimmed.to_string();
            return;
        }
    }

    if let Some(serde_json::Value::Object(ref map)) = row.params {
        if !map.is_empty() {
            let args: Vec<(&str, String)> = map
                .iter()
                .map(|(k, v)| {
                    let s = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    (k.as_str(), s)
                })
                .collect();
            let refs: Vec<(&str, &str)> =
                args.iter().map(|(k, v)| (*k, v.as_str())).collect();
            row.transaction_type_explained = t_args(row.transaction_type.i18n_key(), &refs);
        }
    }
}

fn apply_keyword_filter<'db>(
    query: UserCreditTransactionQuery<'db>,
    value: &str,
) -> UserCreditTransactionQuery<'db> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    if let Ok(clause) = RawClause::new(
        "user_id IN (SELECT id FROM users WHERE username LIKE ?)",
        [pattern],
    ) {
        query.unsafe_sql().where_raw(clause).done()
    } else {
        query
    }
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
