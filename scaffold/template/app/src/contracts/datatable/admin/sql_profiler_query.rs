use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableFilterOptionDto,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest, DataTableScopedContract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.sql_profiler_query";
pub const ROUTE_PREFIX: &str = "/datatable/sql-profiler-query";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct SqlProfilerQueryDatatableRow {
    pub id: String,
    pub request_id: String,
    pub table_name: String,
    pub operation: String,
    pub sql: String,
    pub binds: String,
    pub duration_us: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminSqlProfilerQueryDataTableContract;

impl DataTableScopedContract for AdminSqlProfilerQueryDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = SqlProfilerQueryDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin SQL Profiler Query"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "request_id".to_string(),
                    filter_key: "f-request_id".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Request ID".to_string(),
                    placeholder: Some("UUID".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "table_name".to_string(),
                    filter_key: "f-like-table_name".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Table".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "operation".to_string(),
                    filter_key: "f-operation".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Operation".to_string(),
                    placeholder: Some("All operations".to_string()),
                    description: None,
                    options: Some(operation_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "duration_us_min".to_string(),
                    filter_key: "f-gte-duration_us".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Min Duration (µs)".to_string(),
                    placeholder: Some("e.g. 1000".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "duration_us_max".to_string(),
                    filter_key: "f-lte-duration_us".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Max Duration (µs)".to_string(),
                    placeholder: Some("e.g. 50000".to_string()),
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}

fn operation_filter_options() -> Vec<DataTableFilterOptionDto> {
    ["SELECT", "INSERT", "UPDATE", "DELETE"]
        .iter()
        .map(|op| DataTableFilterOptionDto {
            label: op.to_string(),
            value: op.to_string(),
        })
        .collect()
}
