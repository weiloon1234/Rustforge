use crate::async_export::{DataTableAsyncExportManager, DataTableAsyncExportState};
use crate::executor::execute_datatable;
use crate::filters::{parse_filter_key, ParsedFilter};
use crate::registry::DataTableRegistry;
use crate::traits::{AutoDataTable, BoxFuture, DataTableColumnDescriptor, GeneratedTableAdapter};
use crate::types::{
    DataTableContext, DataTableExecution, DataTableExportMode, DataTableInput,
    DataTablePaginationMode, DataTableUnknownFilterMode, SortDirection,
};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn filter_parser_supports_v1_patterns() {
    assert!(matches!(
        parse_filter_key("f-id"),
        Some(ParsedFilter::Eq { .. })
    ));
    assert!(matches!(
        parse_filter_key("f-like-title"),
        Some(ParsedFilter::Like { .. })
    ));
    assert!(matches!(
        parse_filter_key("f-gte-score"),
        Some(ParsedFilter::Gte { .. })
    ));
    assert!(matches!(
        parse_filter_key("f-has-comments-body"),
        Some(ParsedFilter::Has { .. })
    ));
    assert!(matches!(
        parse_filter_key("f-locale-title"),
        Some(ParsedFilter::LocaleEq { .. })
    ));
    assert!(matches!(
        parse_filter_key("f-has-user__profile-display_name"),
        Some(ParsedFilter::Has { .. })
    ));
    assert!(parse_filter_key("query").is_none());
}

#[derive(Clone, Default)]
struct MockQuery {
    filters: Vec<String>,
    sort: Option<(String, SortDirection)>,
}

#[derive(Clone, Serialize)]
struct MockRow {
    applied_filters: Vec<String>,
    sort_column: Option<String>,
}

struct MockAdapter;

impl GeneratedTableAdapter for MockAdapter {
    type Query<'db> = MockQuery;
    type Row = MockRow;

    fn model_key(&self) -> &'static str {
        "Mock"
    }

    fn sortable_columns(&self) -> &'static [&'static str] {
        &["id", "name"]
    }

    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor] {
        &[
            DataTableColumnDescriptor {
                name: "id",
                label: "Id",
                data_type: "i64",
                sortable: true,
                localized: false,
                filter_ops: &["eq"],
            },
            DataTableColumnDescriptor {
                name: "name",
                label: "Name",
                data_type: "String",
                sortable: true,
                localized: false,
                filter_ops: &["eq", "like"],
            },
        ]
    }

    fn apply_auto_filter<'db>(
        &self,
        mut query: Self::Query<'db>,
        filter: &ParsedFilter,
        _value: &str,
    ) -> anyhow::Result<Option<Self::Query<'db>>> {
        match filter {
            ParsedFilter::Eq { column } if column == "id" => {
                query.filters.push("id".to_string());
                Ok(Some(query))
            }
            _ => Ok(None),
        }
    }

    fn apply_sort<'db>(
        &self,
        mut query: Self::Query<'db>,
        column: &str,
        dir: SortDirection,
    ) -> anyhow::Result<Self::Query<'db>> {
        query.sort = Some((column.to_string(), dir));
        Ok(query)
    }

    fn count<'db>(&self, _query: Self::Query<'db>) -> BoxFuture<'db, anyhow::Result<i64>> {
        Box::pin(async move { Ok(1) })
    }

    fn fetch_page<'db>(
        &self,
        query: Self::Query<'db>,
        _page: i64,
        _per_page: i64,
    ) -> BoxFuture<'db, anyhow::Result<Vec<Self::Row>>> {
        Box::pin(async move {
            Ok(vec![MockRow {
                applied_filters: query.filters,
                sort_column: query.sort.map(|x| x.0),
            }])
        })
    }
}

struct MockTable {
    adapter: MockAdapter,
}

impl AutoDataTable for MockTable {
    type Adapter = MockAdapter;

    fn adapter(&self) -> &Self::Adapter {
        &self.adapter
    }

    fn base_query<'db>(
        &'db self,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> <Self::Adapter as GeneratedTableAdapter>::Query<'db> {
        MockQuery::default()
    }

    fn default_sorting_column(&self) -> &'static str {
        "id"
    }

    fn default_unsortable(&self) -> &'static [&'static str] {
        &["name"]
    }
}

#[tokio::test]
async fn unknown_filter_is_ignored_and_sort_falls_back_to_default() {
    let table = MockTable {
        adapter: MockAdapter,
    };
    let mut input = DataTableInput::from_pairs([
        ("model", "Mock"),
        ("sorting_column", "name"),
        ("sorting", "asc"),
        ("f-id", "1"),
        ("f-unknown", "x"),
    ]);
    input.export = DataTableExportMode::None;

    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let result = execute_datatable(&table, &input, &ctx)
        .await
        .expect("datatable execution should succeed");

    let DataTableExecution::Page(page) = result else {
        panic!("expected page result");
    };

    let first = page
        .records
        .first()
        .and_then(|v| v.as_object())
        .expect("record object");
    let filters = first
        .get("applied_filters")
        .and_then(|v| v.as_array())
        .expect("filters array");
    assert_eq!(filters.len(), 1);
    assert_eq!(filters[0].as_str(), Some("id"));
    assert_eq!(
        first.get("sort_column").and_then(|v| v.as_str()),
        Some("id")
    );
    assert_eq!(page.pagination_mode, DataTablePaginationMode::Offset);
    assert!(page.next_cursor.is_none());
    assert_eq!(page.diagnostics.auto_filters_applied, 1);
    assert!(page.diagnostics.unknown_filters.is_empty());
}

#[tokio::test]
async fn cursor_mode_sets_cursor_metadata() {
    let table = MockTable {
        adapter: MockAdapter,
    };
    let mut input = DataTableInput::from_pairs([
        ("model", "Mock"),
        ("sorting_column", "id"),
        ("sorting", "asc"),
    ]);
    input.export = DataTableExportMode::None;
    input.pagination_mode = DataTablePaginationMode::Cursor;

    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let result = execute_datatable(&table, &input, &ctx)
        .await
        .expect("datatable execution should succeed");

    let DataTableExecution::Page(page) = result else {
        panic!("expected page result");
    };
    assert_eq!(page.pagination_mode, DataTablePaginationMode::Cursor);
    assert!(page.has_more.is_some());
}

#[tokio::test]
async fn unknown_filter_error_mode_rejects_request() {
    let table = MockTable {
        adapter: MockAdapter,
    };
    let mut input = DataTableInput::from_pairs([
        ("model", "Mock"),
        ("sorting_column", "id"),
        ("sorting", "asc"),
        ("f-unknown", "x"),
        ("unknown_filter_mode", "error"),
    ]);
    input.export = DataTableExportMode::None;

    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let err = execute_datatable(&table, &input, &ctx)
        .await
        .expect_err("expected unknown filter rejection");
    assert!(
        err.to_string().contains("Unknown datatable filter"),
        "unexpected error: {}",
        err
    );
}

#[tokio::test]
async fn unknown_filter_warn_mode_keeps_request_and_reports_diagnostics() {
    let table = MockTable {
        adapter: MockAdapter,
    };
    let mut input = DataTableInput::from_pairs([
        ("model", "Mock"),
        ("sorting_column", "id"),
        ("sorting", "asc"),
        ("f-unknown", "x"),
        ("unknown_filter_mode", "warn"),
    ]);
    input.export = DataTableExportMode::None;

    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let result = execute_datatable(&table, &input, &ctx)
        .await
        .expect("datatable execution should succeed");

    let DataTableExecution::Page(page) = result else {
        panic!("expected page result");
    };
    assert_eq!(
        page.diagnostics.unknown_filter_mode,
        DataTableUnknownFilterMode::Warn
    );
    assert_eq!(page.diagnostics.unknown_filters, vec!["f-unknown"]);
}

#[test]
fn describe_exposes_model_defaults_and_columns() {
    let table = MockTable {
        adapter: MockAdapter,
    };
    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let meta = table.describe(&ctx);
    assert_eq!(meta.model, "Mock");
    assert_eq!(meta.defaults.sorting_column, "id");
    assert_eq!(meta.defaults.per_page, 30);
    assert!(meta.columns.iter().any(|c| c.name == "id"));
    assert!(meta.columns.iter().any(|c| c.name == "name"));
}

#[tokio::test]
async fn async_export_manager_completes_csv_job() {
    let mut registry = DataTableRegistry::new();
    registry.register(MockTable {
        adapter: MockAdapter,
    });
    let manager = DataTableAsyncExportManager::new(Arc::new(registry));

    let mut input = DataTableInput::from_pairs([("model", "Mock"), ("sorting_column", "id")]);
    input.export = DataTableExportMode::Csv;

    let ctx = DataTableContext {
        default_per_page: 30,
        app_timezone: "+00:00".to_string(),
        user_timezone: None,
        actor: None,
        unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
    };

    let ticket = manager
        .enqueue(input, ctx)
        .await
        .expect("enqueue should succeed");
    assert_eq!(ticket.state, DataTableAsyncExportState::Queued);

    let mut completed = None;
    for _ in 0..80 {
        if let Some(status) = manager.status(&ticket.job_id).await {
            if matches!(
                status.state,
                DataTableAsyncExportState::Completed | DataTableAsyncExportState::Failed
            ) {
                completed = Some(status);
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let status = completed.expect("export job should finish");
    assert_eq!(status.state, DataTableAsyncExportState::Completed);
    assert!(status.error.is_none());
    let csv = status.csv.expect("csv metadata should exist");
    assert!(Path::new(&csv.file_path).exists());
    let _ = std::fs::remove_file(&csv.file_path);
}
