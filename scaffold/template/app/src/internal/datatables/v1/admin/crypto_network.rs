use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::Op;
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{
        CryptoNetworkDataTable, CryptoNetworkDataTableHooks, CryptoNetworkQuery,
        CryptoNetworkStatus,
    },
    permissions::Permission,
};

use crate::contracts::datatable::admin::crypto_network::{
    AdminCryptoNetworkDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct CryptoNetworkDataTableAppHooks;

impl CryptoNetworkDataTableHooks for CryptoNetworkDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: CryptoNetworkQuery<'db>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> CryptoNetworkQuery<'db> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::CryptoNetworkRead.as_str(),
                Permission::CryptoNetworkManage.as_str(),
            ],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: CryptoNetworkQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<CryptoNetworkQuery<'db>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            "f-status" => {
                if let Some(s) = CryptoNetworkStatus::from_storage(value) {
                    Ok(Some(query.where_status(Op::Eq, s)))
                } else {
                    Ok(Some(query))
                }
            }
            _ => Ok(None),
        }
    }

    fn map_row(
        &self,
        _row: &mut generated::models::CryptoNetworkWithRelations,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn row_to_record(
        &self,
        row: generated::models::CryptoNetworkWithRelations,
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

fn apply_keyword_filter<'db>(
    query: CryptoNetworkQuery<'db>,
    value: &str,
) -> CryptoNetworkQuery<'db> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    if let Ok(id) = trimmed.parse::<i64>() {
        return query.where_id(Op::Eq, id);
    }
    let pattern = format!("%{trimmed}%");
    query.where_name(Op::Like, pattern)
}

pub type AppCryptoNetworkDataTable = CryptoNetworkDataTable<CryptoNetworkDataTableAppHooks>;

pub fn app_crypto_network_datatable(db: sqlx::PgPool) -> AppCryptoNetworkDataTable {
    CryptoNetworkDataTable::new(db).with_hooks(CryptoNetworkDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_crypto_network_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminCryptoNetworkDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
