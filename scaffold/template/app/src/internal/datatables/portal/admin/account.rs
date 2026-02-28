use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::Op;
use core_web::authz::{has_required_permissions, PermissionMode};
use generated::{
    models::{
        AdminCol, AdminDataTable, AdminDataTableConfig, AdminDataTableHooks, AdminQuery, AdminType,
    },
    permissions::Permission,
};

#[derive(Default, Clone)]
pub struct AdminDataTableAppHooks;

impl AdminDataTableHooks for AdminDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: AdminQuery<'db>,
        _input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> AdminQuery<'db> {
        let Some(actor) = ctx.actor.as_ref() else {
            return query.where_id(Op::Eq, -1);
        };

        let admin_type = actor
            .attributes
            .get("admin_type")
            .and_then(|value| value.as_str())
            .and_then(parse_admin_type);

        match admin_type {
            Some(AdminType::Developer) => query,
            Some(AdminType::SuperAdmin) => query.where_admin_type(Op::Ne, AdminType::Developer),
            Some(AdminType::Admin) => query.where_admin_type(Op::Eq, AdminType::Admin),
            None => query.where_id(Op::Eq, -1),
        }
    }

    fn authorize(&self, _input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        Ok(has_required_permissions(
            &actor.permissions,
            &[
                Permission::AdminRead.as_str(),
                Permission::AdminManage.as_str(),
            ],
            PermissionMode::Any,
        ))
    }

    fn filter_query<'db>(
        &'db self,
        query: AdminQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<AdminQuery<'db>>> {
        match filter_key {
            "q" => {
                let pattern = format!("%{value}%");
                Ok(Some(query.where_group(|q| {
                    q.where_col(AdminCol::Username, Op::Like, pattern.clone())
                        .or_where_col(AdminCol::Name, Op::Like, pattern.clone())
                        .or_where_col(AdminCol::Email, Op::Like, pattern)
                })))
            }
            _ => Ok(None),
        }
    }

    fn mappings(
        &self,
        record: &mut serde_json::Map<String, serde_json::Value>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        record.remove("password");
        record.remove("deleted_at");

        if let Some(abilities_val) = record.get("abilities") {
            let strings: Vec<String> = abilities_val
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default();
            record.insert("abilities".to_string(), serde_json::to_value(strings)?);
        }

        Ok(())
    }
}

fn parse_admin_type(value: &str) -> Option<AdminType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "developer" => Some(AdminType::Developer),
        "superadmin" => Some(AdminType::SuperAdmin),
        "admin" => Some(AdminType::Admin),
        _ => None,
    }
}

pub type AppAdminDataTable = AdminDataTable<AdminDataTableAppHooks>;

pub fn app_admin_datatable(db: sqlx::PgPool) -> AppAdminDataTable {
    AdminDataTable::new(db).with_hooks(AdminDataTableAppHooks::default())
}

pub fn app_admin_datatable_with_config(
    db: sqlx::PgPool,
    config: AdminDataTableConfig,
) -> AppAdminDataTable {
    AdminDataTable::new(db)
        .with_hooks(AdminDataTableAppHooks::default())
        .with_config(config)
}

pub fn register_admin_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register(app_admin_datatable(db));
}
