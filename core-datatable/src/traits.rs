use crate::filters::ParsedFilter;
use crate::types::{
    DataTableColumnMeta, DataTableContext, DataTableDefaults, DataTableDescribe, DataTableInput,
    DataTablePaginationMode, DataTableRelationColumnMeta, SortDirection,
};
use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value};
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Copy)]
pub struct DataTableColumnDescriptor {
    pub name: &'static str,
    pub label: &'static str,
    pub data_type: &'static str,
    pub sortable: bool,
    pub localized: bool,
    pub filter_ops: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct DataTableRelationColumnDescriptor {
    pub relation: &'static str,
    pub column: &'static str,
    pub data_type: &'static str,
    pub filter_ops: &'static [&'static str],
}

pub trait GeneratedTableAdapter: Send + Sync + 'static {
    type Query<'db>: Clone + Send + 'db;

    type Row: Serialize + Clone + Send + Sync + 'static;

    fn model_key(&self) -> &'static str;

    fn sortable_columns(&self) -> &'static [&'static str];

    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor];

    fn relation_column_descriptors(&self) -> &'static [DataTableRelationColumnDescriptor] {
        &[]
    }

    fn filter_patterns(&self) -> &'static [&'static str] {
        &[
            "f-<col>",
            "f-like-<col>",
            "f-gte-<col>",
            "f-lte-<col>",
            "f-date-from-<col>",
            "f-date-to-<col>",
            "f-like-any-<col1|col2|...>",
            "f-any-<col1|col2|...>",
            "f-has-<relation>-<col>",
            "f-has-like-<relation>-<col>",
        ]
    }

    fn timestamp_columns(&self) -> &'static [&'static str] {
        &["created_at", "updated_at"]
    }

    fn apply_auto_filter<'db>(
        &self,
        query: Self::Query<'db>,
        filter: &ParsedFilter,
        value: &str,
    ) -> Result<Option<Self::Query<'db>>>
    where
        Self: 'db;

    fn apply_sort<'db>(
        &self,
        query: Self::Query<'db>,
        column: &str,
        dir: SortDirection,
    ) -> Result<Self::Query<'db>>
    where
        Self: 'db;

    fn count<'db>(&self, query: Self::Query<'db>) -> BoxFuture<'db, Result<i64>>
    where
        Self: 'db;

    fn apply_cursor<'db>(
        &self,
        _query: Self::Query<'db>,
        _column: &str,
        _dir: SortDirection,
        _cursor: &str,
    ) -> Result<Option<Self::Query<'db>>>
    where
        Self: 'db,
    {
        Ok(None)
    }

    fn cursor_from_row(&self, _row: &Self::Row, _column: &str) -> Option<String> {
        None
    }

    fn fetch_page<'db>(
        &self,
        query: Self::Query<'db>,
        page: i64,
        per_page: i64,
    ) -> BoxFuture<'db, Result<Vec<Self::Row>>>
    where
        Self: 'db;

    fn row_to_map(&self, row: Self::Row) -> Result<Map<String, Value>> {
        let value = serde_json::to_value(row)?;
        match value {
            Value::Object(map) => Ok(map),
            _ => anyhow::bail!("Generated row must serialize to a JSON object"),
        }
    }
}

pub trait AutoDataTable: Send + Sync + 'static {
    type Adapter: GeneratedTableAdapter;

    fn adapter(&self) -> &Self::Adapter;

    fn model_key(&self) -> &'static str {
        self.adapter().model_key()
    }

    fn base_query<'db>(
        &'db self,
        input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> <Self::Adapter as GeneratedTableAdapter>::Query<'db>;

    fn authorize(&self, _input: &DataTableInput, _ctx: &DataTableContext) -> Result<bool> {
        Ok(true)
    }

    fn filter_query<'db>(
        &'db self,
        _query: <Self::Adapter as GeneratedTableAdapter>::Query<'db>,
        _filter_key: &str,
        _value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Result<Option<<Self::Adapter as GeneratedTableAdapter>::Query<'db>>> {
        Ok(None)
    }

    fn filters<'db>(
        &'db self,
        query: <Self::Adapter as GeneratedTableAdapter>::Query<'db>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Result<<Self::Adapter as GeneratedTableAdapter>::Query<'db>> {
        Ok(query)
    }

    fn mappings(
        &self,
        _record: &mut Map<String, Value>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> Result<()> {
        Ok(())
    }

    fn default_sorting_column(&self) -> &'static str {
        "id"
    }

    fn default_sorted(&self) -> SortDirection {
        SortDirection::Desc
    }

    fn default_export_ignore_columns(&self) -> &'static [&'static str] {
        &["actions", "action"]
    }

    fn default_timestamp_columns(&self) -> &'static [&'static str] {
        self.adapter().timestamp_columns()
    }

    fn default_unsortable(&self) -> &'static [&'static str] {
        &[]
    }

    fn default_row_per_page(&self, ctx: &DataTableContext) -> i64 {
        ctx.default_per_page
    }

    fn describe(&self, ctx: &DataTableContext) -> DataTableDescribe {
        let columns = self
            .adapter()
            .column_descriptors()
            .iter()
            .map(|c| DataTableColumnMeta {
                name: c.name.to_string(),
                label: c.label.to_string(),
                data_type: c.data_type.to_string(),
                sortable: c.sortable,
                localized: c.localized,
                filter_ops: c.filter_ops.iter().map(|s| (*s).to_string()).collect(),
            })
            .collect();
        let relation_columns = self
            .adapter()
            .relation_column_descriptors()
            .iter()
            .map(|c| DataTableRelationColumnMeta {
                relation: c.relation.to_string(),
                column: c.column.to_string(),
                data_type: c.data_type.to_string(),
                filter_ops: c.filter_ops.iter().map(|s| (*s).to_string()).collect(),
            })
            .collect();

        DataTableDescribe {
            model: self.model_key().to_string(),
            defaults: DataTableDefaults {
                sorting_column: self.default_sorting_column().to_string(),
                sorted: self.default_sorted(),
                per_page: self.default_row_per_page(ctx),
                export_ignore_columns: self
                    .default_export_ignore_columns()
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
                timestamp_columns: self
                    .default_timestamp_columns()
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
                unsortable: self
                    .default_unsortable()
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
            },
            columns,
            relation_columns,
            filter_patterns: self
                .adapter()
                .filter_patterns()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }

    fn supports_pagination_mode(&self, mode: DataTablePaginationMode) -> bool {
        match mode {
            DataTablePaginationMode::Offset => true,
            DataTablePaginationMode::Cursor => true,
        }
    }
}
