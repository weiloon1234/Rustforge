use core_datatable::{
    AutoDataTable, BoxFuture, DataTableColumnDescriptor, DataTableContext, DataTableInput,
    DataTableRegistry, GeneratedTableAdapter, SortDirection,
};
use core_db::{
    common::sql::{DbConn, Op, OrderDir},
    generated::models::{Country as CountryModel, CountryCol, CountryQuery, CountryStatus},
};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::permissions::Permission;

use crate::contracts::datatable::admin::country::{
    AdminCountryDataTableContract, CountryDatatableRow, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

const COUNTRY_SORTABLE_COLUMNS: [&str; 7] = [
    "iso2",
    "iso3",
    "name",
    "status",
    "region",
    "calling_code",
    "updated_at",
];

const COUNTRY_COLUMN_DESCRIPTORS: [DataTableColumnDescriptor; 8] = [
    DataTableColumnDescriptor {
        name: "id",
        label: "ID",
        data_type: "string",
        sortable: false,
        localized: false,
        filter_ops: &[],
    },
    DataTableColumnDescriptor {
        name: "iso2",
        label: "ISO2",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq", "like"],
    },
    DataTableColumnDescriptor {
        name: "iso3",
        label: "ISO3",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq", "like"],
    },
    DataTableColumnDescriptor {
        name: "name",
        label: "Name",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["like"],
    },
    DataTableColumnDescriptor {
        name: "region",
        label: "Region",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["like"],
    },
    DataTableColumnDescriptor {
        name: "calling_code",
        label: "Calling Code",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["like"],
    },
    DataTableColumnDescriptor {
        name: "status",
        label: "Status",
        data_type: "string",
        sortable: true,
        localized: false,
        filter_ops: &["eq"],
    },
    DataTableColumnDescriptor {
        name: "updated_at",
        label: "Updated At",
        data_type: "datetime",
        sortable: true,
        localized: false,
        filter_ops: &[],
    },
];

#[derive(Debug, Clone)]
pub struct CountryQueryState {
    keyword: Option<String>,
    status: Option<CountryStatus>,
    region: Option<String>,
    sorting_column: CountryCol,
    sorting: SortDirection,
}

impl Default for CountryQueryState {
    fn default() -> Self {
        Self {
            keyword: None,
            status: None,
            region: None,
            sorting_column: CountryCol::Name,
            sorting: SortDirection::Asc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CountryTableAdapter {
    db: sqlx::PgPool,
}

impl GeneratedTableAdapter for CountryTableAdapter {
    type Query<'db> = CountryQueryState;
    type Row = CountryDatatableRow;

    fn model_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn sortable_columns(&self) -> &'static [&'static str] {
        &COUNTRY_SORTABLE_COLUMNS
    }

    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor] {
        &COUNTRY_COLUMN_DESCRIPTORS
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
        if let Some(normalized) = normalize_sort_column(column) {
            query.sorting_column = normalized;
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
            let base_query = CountryModel::new(DbConn::pool(&db), None).query();
            let filtered_query = apply_country_filters(base_query, &query);
            Ok(filtered_query.count().await?)
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

            let base_query = CountryModel::new(DbConn::pool(&db), None).query();
            let filtered_query = apply_country_filters(base_query, &query)
                .order_by(query.sorting_column, to_order_direction(query.sorting))
                .order_by(CountryCol::Iso2, OrderDir::Asc)
                .offset(offset)
                .limit(safe_per_page);

            let rows = filtered_query.get().await?;
            let out = rows
                .into_iter()
                .map(|row| CountryDatatableRow {
                    id: row.iso2.clone(),
                    iso2: row.iso2,
                    iso3: row.iso3,
                    name: row.name,
                    region: row.region,
                    calling_code: row.calling_code,
                    status: row.status.as_str().to_string(),
                    updated_at: format_rfc3339(row.updated_at),
                })
                .collect();

            Ok(out)
        })
    }
}

#[derive(Default, Clone)]
pub struct CountryDataTableAppHooks;

impl CountryDataTableAppHooks {
    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };

        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::CountryRead.as_str(),
                Permission::CountryManage.as_str(),
            ],
            PermissionMode::Any,
        );

        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        mut query: CountryQueryState,
        filter_key: &str,
        value: &str,
    ) -> Option<CountryQueryState> {
        match filter_key {
            "q" => {
                query.keyword = to_non_empty(value).map(ToString::to_string);
                Some(query)
            }
            "status" => {
                query.status = to_non_empty(value).and_then(normalize_country_status);
                Some(query)
            }
            "region" => {
                query.region = to_non_empty(value).map(ToString::to_string);
                Some(query)
            }
            _ => None,
        }
    }
}

pub struct CountryDataTable {
    adapter: CountryTableAdapter,
    hooks: CountryDataTableAppHooks,
}

impl CountryDataTable {
    fn new(db: sqlx::PgPool) -> Self {
        Self {
            adapter: CountryTableAdapter { db },
            hooks: CountryDataTableAppHooks,
        }
    }
}

impl AutoDataTable for CountryDataTable {
    type Adapter = CountryTableAdapter;

    fn adapter(&self) -> &Self::Adapter {
        &self.adapter
    }

    fn base_query<'db>(
        &'db self,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> <Self::Adapter as GeneratedTableAdapter>::Query<'db> {
        CountryQueryState::default()
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
        "name"
    }

    fn default_sorted(&self) -> SortDirection {
        SortDirection::Asc
    }

    fn default_timestamp_columns(&self) -> &'static [&'static str] {
        &["updated_at"]
    }
}

pub type AppCountryDataTable = CountryDataTable;

pub fn app_country_datatable(db: sqlx::PgPool) -> AppCountryDataTable {
    CountryDataTable::new(db)
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_country_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminCountryDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}

fn apply_country_filters<'db>(
    mut query: CountryQuery<'db>,
    filters: &CountryQueryState,
) -> CountryQuery<'db> {
    if let Some(keyword) = filters.keyword.as_deref().and_then(to_non_empty) {
        let pattern = format!("%{}%", keyword.trim());
        query = query
            .where_col(CountryCol::Iso2, Op::ILike, pattern.clone())
            .or_where_col(CountryCol::Iso3, Op::ILike, pattern.clone())
            .or_where_col(CountryCol::Name, Op::ILike, pattern.clone())
            .or_where_col(CountryCol::CallingCode, Op::ILike, pattern);
    }

    if let Some(status) = filters.status {
        query = query.where_status(Op::Eq, status);
    }

    if let Some(region) = filters.region.as_deref().and_then(to_non_empty) {
        query = query.where_col(
            CountryCol::Region,
            Op::ILike,
            format!("%{}%", region.trim()),
        );
    }

    query
}

fn to_order_direction(direction: SortDirection) -> OrderDir {
    match direction {
        SortDirection::Asc => OrderDir::Asc,
        SortDirection::Desc => OrderDir::Desc,
    }
}

fn to_non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn normalize_sort_column(value: &str) -> Option<CountryCol> {
    match value.trim().to_ascii_lowercase().as_str() {
        "iso2" => Some(CountryCol::Iso2),
        "iso3" => Some(CountryCol::Iso3),
        "name" => Some(CountryCol::Name),
        "status" => Some(CountryCol::Status),
        "region" => Some(CountryCol::Region),
        "calling_code" => Some(CountryCol::CallingCode),
        "updated_at" => Some(CountryCol::UpdatedAt),
        _ => None,
    }
}

fn format_rfc3339(value: time::OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

fn normalize_country_status(value: &str) -> Option<CountryStatus> {
    CountryStatus::from_storage(value)
}
