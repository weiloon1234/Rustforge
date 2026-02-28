use crate::csv::{value_to_csv, write_csv_row};
use crate::filters::parse_filter_key;
use crate::timezone::apply_timezone_to_fields;
use crate::traits::{AutoDataTable, GeneratedTableAdapter};
use crate::types::{
    DataTableContext, DataTableCsvExport, DataTableDiagnostics, DataTableExecution,
    DataTableExportMode, DataTableInput, DataTablePaginationMode, DataTableResult,
    DataTableUnknownFilterMode,
};
use anyhow::Result;
use core_db::common::pagination::resolve_per_page;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;
use uuid::Uuid;

pub async fn execute_datatable<T: AutoDataTable>(
    table: &T,
    input: &DataTableInput,
    ctx: &DataTableContext,
) -> Result<DataTableExecution> {
    let started = Instant::now();

    if !table.authorize(input, ctx)? {
        anyhow::bail!("Permission denied");
    }

    let adapter = table.adapter();
    let mut query = table.base_query(input, ctx);
    let unknown_filter_mode = input
        .unknown_filter_mode()
        .unwrap_or(ctx.unknown_filter_mode);
    let mut unknown_filters = Vec::new();
    let mut auto_filters_applied = 0usize;

    for (key, raw_value) in input.filter_entries() {
        let Some(filter) = parse_filter_key(key) else {
            continue;
        };
        if let Some(next) = adapter.apply_auto_filter(query.clone(), &filter, raw_value)? {
            query = next;
            auto_filters_applied += 1;
        } else {
            unknown_filters.push(key.to_string());
            if matches!(unknown_filter_mode, DataTableUnknownFilterMode::Error) {
                anyhow::bail!(
                    "Unknown datatable filter '{}' for model '{}'",
                    key,
                    table.model_key()
                );
            }
        }
    }

    for (key, raw_value) in input.custom_filter_entries() {
        if let Some(next) = table.filter_query(query.clone(), key, raw_value, input, ctx)? {
            query = next;
        } else {
            unknown_filters.push(key.to_string());
        }
    }

    query = table.filters(query, input, ctx)?;

    let requested_col = input
        .sorting_column
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(str::trim);

    let effective_sort_col = resolve_sort_column(
        requested_col,
        table.default_sorting_column(),
        table.default_unsortable(),
        adapter.sortable_columns(),
    );

    let sort_dir = input.sorting.unwrap_or_else(|| table.default_sorted());
    query = adapter.apply_sort(query, effective_sort_col, sort_dir)?;

    let requested_per_page = if input.ipp > 0 {
        input.ipp
    } else {
        table.default_row_per_page(ctx)
    };
    let per_page = resolve_per_page(requested_per_page);

    if !table.supports_pagination_mode(input.pagination_mode) {
        anyhow::bail!(
            "Pagination mode '{:?}' is not supported by model '{}'",
            input.pagination_mode,
            table.model_key()
        );
    }

    let make_diagnostics = || DataTableDiagnostics {
        duration_ms: started.elapsed().as_millis() as u64,
        auto_filters_applied,
        unknown_filters: if matches!(unknown_filter_mode, DataTableUnknownFilterMode::Ignore) {
            vec![]
        } else {
            unknown_filters.clone()
        },
        unknown_filter_mode,
    };

    match input.export {
        DataTableExportMode::None => {
            let (rows, total_records, total_pages, page, has_more, next_cursor, pagination_mode) =
                match input.pagination_mode {
                    DataTablePaginationMode::Offset => {
                        let total_records = adapter.count(query.clone()).await?;
                        let total_pages = ((total_records + per_page - 1) / per_page).max(1);
                        let page = input.page.max(1).min(total_pages);
                        let rows = adapter.fetch_page(query, page, per_page).await?;
                        (
                            rows,
                            total_records,
                            total_pages,
                            page,
                            None,
                            None,
                            DataTablePaginationMode::Offset,
                        )
                    }
                    DataTablePaginationMode::Cursor => {
                        let query = if let Some(cursor) = input.cursor.as_deref() {
                            match adapter.apply_cursor(
                                query,
                                effective_sort_col,
                                sort_dir,
                                cursor.trim(),
                            )? {
                                Some(q) => q,
                                None => anyhow::bail!(
                                    "Cursor mode is not supported for column '{}'",
                                    effective_sort_col
                                ),
                            }
                        } else {
                            query
                        };
                        let mut rows = adapter.fetch_page(query, 1, per_page + 1).await?;
                        let has_more = rows.len() > per_page as usize;
                        if has_more {
                            rows.truncate(per_page as usize);
                        }
                        let next_cursor = if has_more {
                            rows.last()
                                .and_then(|r| adapter.cursor_from_row(r, effective_sort_col))
                        } else {
                            None
                        };
                        (
                            rows,
                            0,
                            0,
                            1,
                            Some(has_more),
                            next_cursor,
                            DataTablePaginationMode::Cursor,
                        )
                    }
                };
            let tz = ctx.effective_timezone(input.timezone.as_deref());
            let timestamp_columns = table
                .default_timestamp_columns()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            let mut records = Vec::with_capacity(rows.len());
            for row in rows {
                let mut record = adapter.row_to_map(row)?;
                apply_timezone_to_fields(&mut record, &timestamp_columns, &tz);
                table.mappings(&mut record, input, ctx)?;
                records.push(Value::Object(record));
            }

            Ok(DataTableExecution::Page(DataTableResult {
                records,
                per_page,
                total_records,
                total_pages,
                page,
                pagination_mode,
                has_more,
                next_cursor,
                diagnostics: make_diagnostics(),
            }))
        }
        DataTableExportMode::Csv => {
            let total_records = adapter.count(query.clone()).await?;
            let export_per_page = per_page.max(1);
            let total_pages = ((total_records + export_per_page - 1) / export_per_page).max(1);

            let file_name = input
                .export_file_name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("datatable-{}.csv", Uuid::new_v4()));

            let file_path = std::env::temp_dir().join(format!(
                "rs-core-dt-{}-{}.csv",
                table.model_key(),
                Uuid::new_v4()
            ));

            let file = File::create(&file_path)?;
            let mut writer = BufWriter::new(file);

            let tz = ctx.effective_timezone(input.timezone.as_deref());
            let timestamp_columns = table
                .default_timestamp_columns()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            let ignore = table
                .default_export_ignore_columns()
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>();
            let header_spec = parse_export_headers(input, &ignore)?;

            let mut wrote_header = false;
            let mut columns: Vec<String> = vec![];
            let mut labels: Vec<String> = vec![];

            for page in 1..=total_pages {
                let rows = adapter
                    .fetch_page(query.clone(), page, export_per_page)
                    .await?;
                if rows.is_empty() {
                    continue;
                }

                for row in rows {
                    let mut record = adapter.row_to_map(row)?;
                    apply_timezone_to_fields(&mut record, &timestamp_columns, &tz);
                    table.mappings(&mut record, input, ctx)?;

                    if !wrote_header {
                        if let Some(spec) = &header_spec {
                            columns = spec.columns.clone();
                            labels = spec.labels.clone();
                        } else {
                            columns = record.keys().cloned().collect();
                            labels = columns.clone();
                        }
                        write_csv_row(&mut writer, &labels)?;
                        wrote_header = true;
                    }

                    let values = columns
                        .iter()
                        .map(|col| value_to_csv(record.get(col)))
                        .collect::<Vec<_>>();
                    write_csv_row(&mut writer, &values)?;
                }
            }

            if !wrote_header {
                if let Some(spec) = &header_spec {
                    write_csv_row(&mut writer, &spec.labels)?;
                }
            }

            Ok(DataTableExecution::Csv(DataTableCsvExport {
                file_path: file_path.to_string_lossy().to_string(),
                file_name,
                content_type: "text/csv; charset=utf-8".to_string(),
                total_records,
                diagnostics: make_diagnostics(),
            }))
        }
    }
}

fn resolve_sort_column<'a>(
    requested: Option<&str>,
    default_col: &'a str,
    unsortable: &[&str],
    sortable: &'a [&'a str],
) -> &'a str {
    let mut chosen = default_col;

    if let Some(col) = requested {
        let is_unsortable = unsortable.iter().any(|x| x.eq_ignore_ascii_case(col));
        let is_sortable = sortable.iter().any(|x| x.eq_ignore_ascii_case(col));
        if !is_unsortable && is_sortable {
            for known in sortable {
                if known.eq_ignore_ascii_case(col) {
                    return known;
                }
            }
        }
    }

    for known in sortable {
        if known.eq_ignore_ascii_case(default_col) {
            chosen = known;
            break;
        }
    }

    chosen
}

#[derive(Debug, Clone)]
struct ExportHeaderSpec {
    labels: Vec<String>,
    columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct HeaderInput {
    label: Option<String>,
    column: Option<String>,
    export_ignore: Option<bool>,
    export_column: Option<String>,
}

fn parse_export_headers(
    input: &DataTableInput,
    ignore_defaults: &HashSet<String>,
) -> Result<Option<ExportHeaderSpec>> {
    let Some(raw) = input.params.get("headers") else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let items: Vec<HeaderInput> = serde_json::from_str(raw)?;
    let mut labels = Vec::new();
    let mut columns = Vec::new();

    for item in items {
        if item.export_ignore.unwrap_or(false) {
            continue;
        }

        let column = item
            .export_column
            .or(item.column)
            .unwrap_or_default()
            .trim()
            .to_string();
        if column.is_empty() || ignore_defaults.contains(&column) {
            continue;
        }

        labels.push(item.label.unwrap_or_else(|| column.clone()));
        columns.push(column);
    }

    if columns.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ExportHeaderSpec { labels, columns }))
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_sort_column;

    #[test]
    fn sort_column_falls_back_when_invalid() {
        let sortable = ["id", "name"];
        let unsortable = ["name"];

        assert_eq!(
            resolve_sort_column(Some("name"), "id", &unsortable, &sortable),
            "id"
        );
        assert_eq!(
            resolve_sort_column(Some("unknown"), "id", &unsortable, &sortable),
            "id"
        );
        assert_eq!(
            resolve_sort_column(Some("NAME"), "id", &[], &sortable),
            "name"
        );
    }
}
