use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::Op;
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::models::{
    AuditAction, AuditLog, AuditLogCol, AuditLogDataTable, AuditLogDataTableHooks, AuditLogQuery,
};
use generated::permissions::Permission;

use crate::contracts::datatable::admin::audit_log::{
    AdminAuditLogDataTableContract, AuditLogDatatableSummaryOutput, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct AuditLogDataTableAppHooks;

impl AuditLogDataTableHooks for AuditLogDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: AuditLogQuery<'db>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> AuditLogQuery<'db> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[Permission::AuditLogRead.as_str()],
            PermissionMode::Any,
        );
        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: AuditLogQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<AuditLogQuery<'db>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            _ => Ok(None),
        }
    }

    fn row_to_record(
        &self,
        row: generated::models::AuditLogView,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        self.default_row_to_record(row)
    }
}

fn apply_keyword_filter<'db>(query: AuditLogQuery<'db>, value: &str) -> AuditLogQuery<'db> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    query.where_group(|q| {
        q.where_col(AuditLogCol::OldData, Op::Like, pattern.clone())
            .or_where_col(AuditLogCol::NewData, Op::Like, pattern.clone())
            .or_where_col(AuditLogCol::TableName, Op::Like, pattern)
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
    mut query: AuditLogQuery<'db>,
    input: &DataTableInput,
) -> AuditLogQuery<'db> {
    for (key, value) in input.filter_entries() {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        match key {
            "f-table_name" => {
                query = query.where_col(AuditLogCol::TableName, Op::Eq, trimmed.to_string());
            }
            "f-action" => {
                if let Some(action) = AuditAction::from_storage(trimmed) {
                    query = query.where_action(Op::Eq, action);
                }
            }
            "f-record_id" => {
                if let Ok(id) = trimmed.parse::<i64>() {
                    query = query.where_record_id(Op::Eq, id);
                }
            }
            "f-date-from-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, false) {
                    query = query.where_col(AuditLogCol::CreatedAt, Op::Ge, ts);
                }
            }
            "f-date-to-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, true) {
                    query = query.where_col(AuditLogCol::CreatedAt, Op::Le, ts);
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

pub async fn build_audit_log_summary_output(
    db: &sqlx::PgPool,
    input: &DataTableInput,
    _ctx: &DataTableContext,
) -> anyhow::Result<AuditLogDatatableSummaryOutput> {
    let filtered = apply_summary_filters(AuditLog::new(db, None).query(), input);

    let total_filtered = filtered.clone().count().await?;
    let create_count = filtered
        .clone()
        .where_action(Op::Eq, AuditAction::Create)
        .count()
        .await?;
    let update_count = filtered
        .clone()
        .where_action(Op::Eq, AuditAction::Update)
        .count()
        .await?;
    let delete_count = filtered
        .where_action(Op::Eq, AuditAction::Delete)
        .count()
        .await?;

    Ok(AuditLogDatatableSummaryOutput {
        total_filtered,
        create_count,
        update_count,
        delete_count,
    })
}

pub type AppAuditLogDataTable = AuditLogDataTable<AuditLogDataTableAppHooks>;

pub fn app_audit_log_datatable(db: sqlx::PgPool) -> AppAuditLogDataTable {
    AuditLogDataTable::new(db).with_hooks(AuditLogDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_audit_log_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminAuditLogDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
