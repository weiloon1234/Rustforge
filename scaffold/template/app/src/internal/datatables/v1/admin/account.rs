use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::{model_api::Query, sql::Op};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::*,
    permissions::Permission,
};

use crate::contracts::datatable::admin::account::{
    AdminAdminDataTableContract, AdminDatatableSummaryOutput, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct AdminDataTableAppHooks;

impl AdminDataTableHooks for AdminDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: Query<'db, AdminModel>,
        _input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> Query<'db, AdminModel> {
        apply_actor_scope(query, ctx)
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::AdminRead.as_str(),
                Permission::AdminManage.as_str(),
            ],
            PermissionMode::Any,
        );

        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: Query<'db, AdminModel>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<Query<'db, AdminModel>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            _ => Ok(None),
        }
    }

    fn row_to_record(
        &self,
        row: generated::models::AdminRecord,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let identity = row.identity();
        let mut record = self.default_row_to_record(row)?;
        record.insert("identity".to_string(), serde_json::Value::String(identity));
        record.remove("password");
        record.remove("deleted_at");
        Ok(record)
    }
}

fn parse_admin_type(value: &str) -> Option<AdminType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "developer" => Some(AdminType::Developer),
        "superadmin" | "super_admin" | "super-admin" => Some(AdminType::SuperAdmin),
        "admin" => Some(AdminType::Admin),
        _ => None,
    }
}

fn apply_actor_scope<'db>(query: Query<'db, AdminModel>, ctx: &DataTableContext) -> Query<'db, AdminModel> {
    let Some(actor) = ctx.actor.as_ref() else {
        return query.where_col(AdminCol::ID, Op::Eq, -1);
    };

    let admin_type = actor
        .attributes
        .get("admin_type")
        .and_then(|value| value.as_str())
        .and_then(parse_admin_type);

    match admin_type {
        Some(AdminType::Developer) => query,
        Some(AdminType::SuperAdmin) => query.where_col(AdminCol::ADMIN_TYPE, Op::Ne, AdminType::Developer),
        Some(AdminType::Admin) => query.where_col(AdminCol::ADMIN_TYPE, Op::Eq, AdminType::Admin),
        None => query.where_col(AdminCol::ID, Op::Eq, -1),
    }
}

fn apply_keyword_filter<'db>(query: Query<'db, AdminModel>, value: &str) -> Query<'db, AdminModel> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    query.where_group(|q| {
        q.where_col(AdminCol::USERNAME, Op::Like, pattern.clone())
            .or_where_col(AdminCol::NAME, Op::Like, pattern.clone())
            .or_where_col(AdminCol::EMAIL, Op::Like, pattern)
    })
}

fn parse_datetime(raw: &str, end_of_day: bool) -> Option<time::OffsetDateTime> {
    let trimmed = raw.trim();
    if let Ok(dt) =
        time::OffsetDateTime::parse(trimmed, &time::format_description::well_known::Rfc3339)
    {
        return Some(dt);
    }
    if trimmed.len() == 10 {
        let date = time::Date::parse(
            trimmed,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .ok()?;
        let t = if end_of_day {
            time::Time::from_hms(23, 59, 59).ok()?
        } else {
            time::Time::MIDNIGHT
        };
        return Some(date.with_time(t).assume_offset(time::UtcOffset::UTC));
    }
    None
}

fn apply_summary_filters<'db>(
    mut query: Query<'db, AdminModel>,
    input: &DataTableInput,
) -> Query<'db, AdminModel> {
    for (key, value) in input.filter_entries() {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        match key {
            "f-like-email" => {
                query = query.where_col(AdminCol::EMAIL, Op::Like, format!("%{trimmed}%"));
            }
            "f-like-username" => {
                query = query.where_col(AdminCol::USERNAME, Op::Like, format!("%{trimmed}%"));
            }
            "f-admin_type" => {
                if let Some(admin_type) = parse_admin_type(trimmed) {
                    query = query.where_col(AdminCol::ADMIN_TYPE, Op::Eq, admin_type);
                }
            }
            "f-date-from-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, false) {
                    query = query.where_col(AdminCol::CREATED_AT, Op::Ge, ts);
                }
            }
            "f-date-to-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, true) {
                    query = query.where_col(AdminCol::CREATED_AT, Op::Le, ts);
                }
            }
            _ => {}
        }
    }

    for (key, value) in input.custom_filter_entries() {
        if key == "q" {
            query = apply_keyword_filter(query, value);
        }
    }

    query
}

pub async fn build_admin_summary_output(
    db: &sqlx::PgPool,
    input: &DataTableInput,
    ctx: &DataTableContext,
) -> anyhow::Result<AdminDatatableSummaryOutput> {
    let scoped = apply_actor_scope(AdminModel::query(db), ctx);
    let filtered = apply_summary_filters(scoped, input);

    let total_filtered = filtered.clone().count().await?;
    let developer_count = filtered
        .clone()
        .where_col(AdminCol::ADMIN_TYPE, Op::Eq, AdminType::Developer)
        .count()
        .await?;
    let superadmin_count = filtered
        .clone()
        .where_col(AdminCol::ADMIN_TYPE, Op::Eq, AdminType::SuperAdmin)
        .count()
        .await?;
    let admin_count = filtered
        .where_col(AdminCol::ADMIN_TYPE, Op::Eq, AdminType::Admin)
        .count()
        .await?;

    Ok(AdminDatatableSummaryOutput {
        total_admin_counts: total_filtered,
        total_filtered,
        developer_count,
        superadmin_count,
        admin_count,
    })
}

pub type AppAdminDataTable = AdminDataTable<AdminDataTableAppHooks>;

pub fn app_admin_datatable(db: sqlx::PgPool) -> AppAdminDataTable {
    AdminDataTable::new(db).with_hooks(AdminDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_admin_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminAdminDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
