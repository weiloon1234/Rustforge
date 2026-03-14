use core_datatable::{
    AutoDataTable, BoxFuture, DataTableColumnDescriptor, DataTableContext, DataTableInput,
    DataTableRegistry, GeneratedTableAdapter, SortDirection,
};
use core_db::common::{
    model_api::Query,
    sql::{DbConn, Op, OrderDir, RawClause},
};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::models::{
    AuditAction, AuditLogCol, AuditLogModel, AuditLogRecord,
};
use generated::permissions::Permission;

use crate::contracts::datatable::admin::audit_log::{
    AdminAuditLogDataTableContract, AuditLogDatatableRow,
    ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

const AUDIT_LOG_SORTABLE_COLUMNS: [&str; 6] = [
    "id",
    "admin_id",
    "action",
    "table_name",
    "record_key",
    "created_at",
];

const AUDIT_LOG_COLUMN_DESCRIPTORS: [DataTableColumnDescriptor; 9] = [
    DataTableColumnDescriptor {
        name: "id",
        label: "ID",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "admin_id",
        label: "Admin ID",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "admin_username",
        label: "Admin",
        data_type: "string",
        sortable: false,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "action",
        label: "Action",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq"],
    },
    DataTableColumnDescriptor {
        name: "table_name",
        label: "Table Name",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq", "like"],
    },
    DataTableColumnDescriptor {
        name: "record_key",
        label: "Record Key",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq"],
    },
    DataTableColumnDescriptor {
        name: "old_data",
        label: "Old Data",
        data_type: "json",
        sortable: false,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "new_data",
        label: "New Data",
        data_type: "json",
        sortable: false,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "created_at",
        label: "Created At",
        data_type: "datetime",
        sortable: true,
        localized: false,
        filter_ops: &[],
    },
];

#[derive(Debug, Clone)]
pub struct AuditLogQueryState {
    keyword: Option<String>,
    table_name: Option<String>,
    action: Option<AuditAction>,
    record_key: Option<String>,
    date_from: Option<time::OffsetDateTime>,
    date_to: Option<time::OffsetDateTime>,
    sorting_column: &'static str,
    sorting: SortDirection,
}

impl Default for AuditLogQueryState {
    fn default() -> Self {
        Self {
            keyword: None,
            table_name: None,
            action: None,
            record_key: None,
            date_from: None,
            date_to: None,
            sorting_column: "id",
            sorting: SortDirection::Desc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditLogTableAdapter {
    db: sqlx::PgPool,
}

impl GeneratedTableAdapter for AuditLogTableAdapter {
    type Query<'db> = AuditLogQueryState;
    type Row = AuditLogDatatableRow;

    fn model_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn sortable_columns(&self) -> &'static [&'static str] {
        &AUDIT_LOG_SORTABLE_COLUMNS
    }

    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor] {
        &AUDIT_LOG_COLUMN_DESCRIPTORS
    }

    fn apply_auto_filter<'db>(
        &self,
        _query: Self::Query<'db>,
        _filter: &core_datatable::ParsedFilter,
        _value: &str,
    ) -> anyhow::Result<Option<Self::Query<'db>>>
    where
        Self: 'db,
    {
        Ok(None)
    }

    fn apply_sort<'db>(
        &self,
        mut query: Self::Query<'db>,
        column: &str,
        dir: SortDirection,
    ) -> anyhow::Result<Self::Query<'db>>
    where
        Self: 'db,
    {
        if let Some(key) = normalize_sort_column(column) {
            query.sorting_column = key;
        }
        query.sorting = dir;
        Ok(query)
    }

    fn count<'db>(&self, query: Self::Query<'db>) -> BoxFuture<'db, anyhow::Result<i64>>
    where
        Self: 'db,
    {
        let db = self.db.clone();
        Box::pin(async move {
            let base = AuditLogModel::query(DbConn::pool(&db));
            let filtered = apply_audit_log_filters(base, &query);
            Ok(filtered.count().await?)
        })
    }

    fn fetch_page<'db>(
        &self,
        query: Self::Query<'db>,
        page: i64,
        per_page: i64,
    ) -> BoxFuture<'db, anyhow::Result<Vec<Self::Row>>>
    where
        Self: 'db,
    {
        let db = self.db.clone();
        Box::pin(async move {
            let safe_page = page.max(1);
            let safe_per_page = per_page.max(1);
            let offset = (safe_page - 1) * safe_per_page;

            let base = AuditLogModel::query(DbConn::pool(&db));
            let filtered = apply_default_audit_log_sort(apply_audit_log_sort(
                apply_audit_log_filters(base, &query),
                query.sorting_column,
                to_order_direction(query.sorting),
            ))
            .offset(offset)
            .limit(safe_per_page);

            let rows = filtered.all().await?;

            let out = rows
                .into_iter()
                .map(|r: AuditLogRecord| {
                    let admin_username = r.admin.as_ref()
                        .map(|a| a.username.clone())
                        .unwrap_or_else(|| r.admin_id.to_string());
                    AuditLogDatatableRow {
                        id: r.id.into(),
                        admin_id: r.admin_id,
                        admin_username,
                        action: r.action,
                        action_explained: r.action_explained,
                        table_name: r.table_name,
                        record_key: r.record_key,
                        old_data: r.old_data,
                        new_data: r.new_data,
                        created_at: format_rfc3339(r.created_at),
                    }
                })
                .collect();

            Ok(out)
        })
    }
}

#[derive(Default, Clone)]
pub struct AuditLogDataTableAppHooks;

impl AuditLogDataTableAppHooks {
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

    fn filter_query(
        &self,
        mut query: AuditLogQueryState,
        filter_key: &str,
        value: &str,
    ) -> Option<AuditLogQueryState> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Some(query);
        }
        match filter_key {
            "q" => {
                query.keyword = Some(trimmed.to_string());
                Some(query)
            }
            "f-table_name" => {
                query.table_name = Some(trimmed.to_string());
                Some(query)
            }
            "f-action" => {
                query.action = AuditAction::from_storage(trimmed);
                Some(query)
            }
            "f-record_key" => {
                query.record_key = Some(trimmed.to_string());
                Some(query)
            }
            "f-date-from-created_at" => {
                query.date_from = parse_datetime(trimmed, false);
                Some(query)
            }
            "f-date-to-created_at" => {
                query.date_to = parse_datetime(trimmed, true);
                Some(query)
            }
            _ => None,
        }
    }
}

pub struct AuditLogDataTable {
    adapter: AuditLogTableAdapter,
    hooks: AuditLogDataTableAppHooks,
}

impl AuditLogDataTable {
    fn new(db: sqlx::PgPool) -> Self {
        Self {
            adapter: AuditLogTableAdapter { db },
            hooks: AuditLogDataTableAppHooks,
        }
    }
}

impl AutoDataTable for AuditLogDataTable {
    type Adapter = AuditLogTableAdapter;

    fn adapter(&self) -> &Self::Adapter {
        &self.adapter
    }

    fn base_query<'db>(
        &'db self,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> <Self::Adapter as GeneratedTableAdapter>::Query<'db> {
        AuditLogQueryState::default()
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        self.hooks.authorize(input, ctx)
    }

    fn filter_query<'db>(
        &'db self,
        query: <Self::Adapter as GeneratedTableAdapter>::Query<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<<Self::Adapter as GeneratedTableAdapter>::Query<'db>>> {
        Ok(self.hooks.filter_query(query, filter_key, value))
    }

    fn default_sorting_column(&self) -> &'static str {
        "id"
    }

    fn default_sorted(&self) -> SortDirection {
        SortDirection::Desc
    }

    fn default_timestamp_columns(&self) -> &'static [&'static str] {
        &["created_at"]
    }
}

pub type AppAuditLogDataTable = AuditLogDataTable;

pub fn app_audit_log_datatable(db: sqlx::PgPool) -> AppAuditLogDataTable {
    AuditLogDataTable::new(db)
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

fn apply_audit_log_filters<'db>(
    mut query: Query<'db, AuditLogModel>,
    filters: &AuditLogQueryState,
) -> Query<'db, AuditLogModel> {
    if let Some(keyword) = filters.keyword.as_deref() {
        let trimmed = keyword.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{trimmed}%");
            if let Ok(clause) = RawClause::new(
                "(old_data::text ILIKE ? OR new_data::text ILIKE ? OR table_name ILIKE ?)",
                [pattern.clone(), pattern.clone(), pattern],
            ) {
                let (sql, binds) = clause.into_parts();
                query = query.unsafe_sql().where_raw(sql, binds).done();
            }
        }
    }

    if let Some(table_name) = &filters.table_name {
        query = query.where_col(AuditLogCol::TABLE_NAME, Op::Eq, table_name.clone());
    }

    if let Some(action) = filters.action {
        query = query.where_col(AuditLogCol::ACTION, Op::Eq, action);
    }

    if let Some(record_key) = &filters.record_key {
        query = query.where_col(AuditLogCol::RECORD_KEY, Op::Eq, record_key.clone());
    }

    if let Some(ts) = filters.date_from {
        query = query.where_col(AuditLogCol::CREATED_AT, Op::Ge, ts);
    }

    if let Some(ts) = filters.date_to {
        query = query.where_col(AuditLogCol::CREATED_AT, Op::Le, ts);
    }

    query
}

fn to_order_direction(direction: SortDirection) -> OrderDir {
    match direction {
        SortDirection::Asc => OrderDir::Asc,
        SortDirection::Desc => OrderDir::Desc,
    }
}

fn normalize_sort_column(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "id" => Some("id"),
        "admin_id" => Some("admin_id"),
        "action" => Some("action"),
        "table_name" => Some("table_name"),
        "record_key" => Some("record_key"),
        "created_at" => Some("created_at"),
        _ => None,
    }
}

fn apply_audit_log_sort<'db>(
    query: Query<'db, AuditLogModel>,
    sorting_column: &str,
    direction: OrderDir,
) -> Query<'db, AuditLogModel> {
    match sorting_column {
        "admin_id" => query.order_by(AuditLogCol::ADMIN_ID, direction),
        "action" => query.order_by(AuditLogCol::ACTION, direction),
        "table_name" => query.order_by(AuditLogCol::TABLE_NAME, direction),
        "record_key" => query.order_by(AuditLogCol::RECORD_KEY, direction),
        "created_at" => query.order_by(AuditLogCol::CREATED_AT, direction),
        _ => query.order_by(AuditLogCol::ID, direction),
    }
}

fn apply_default_audit_log_sort<'db>(
    query: Query<'db, AuditLogModel>,
) -> Query<'db, AuditLogModel> {
    query.order_by(AuditLogCol::ID, OrderDir::Desc)
}

fn format_rfc3339(value: time::OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
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
