use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableFilterOptionDto,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest, DataTableScopedContract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.sql_profiler_request";
pub const ROUTE_PREFIX: &str = "/datatable/sql-profiler-request";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct SqlProfilerRequestDatatableRow {
    pub id: String,
    pub request_method: String,
    pub request_path: String,
    pub total_queries: i32,
    pub total_duration_ms: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminSqlProfilerRequestDataTableContract;

impl DataTableScopedContract for AdminSqlProfilerRequestDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = SqlProfilerRequestDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin SQL Profiler Request"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "id".to_string(),
                    filter_key: "f-id".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Request ID".to_string(),
                    placeholder: Some("UUID".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "request_path".to_string(),
                    filter_key: "f-like-request_path".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Path".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "request_method".to_string(),
                    filter_key: "f-request_method".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Method".to_string(),
                    placeholder: Some("All methods".to_string()),
                    description: None,
                    options: Some(method_filter_options()),
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "total_queries_min".to_string(),
                    filter_key: "f-gte-total_queries".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Min Queries".to_string(),
                    placeholder: Some("e.g. 5".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "total_duration_ms_min".to_string(),
                    filter_key: "f-gte-total_duration_ms".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Min Duration (ms)".to_string(),
                    placeholder: Some("e.g. 100".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "total_duration_ms_max".to_string(),
                    filter_key: "f-lte-total_duration_ms".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Max Duration (ms)".to_string(),
                    placeholder: Some("e.g. 5000".to_string()),
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}

fn method_filter_options() -> Vec<DataTableFilterOptionDto> {
    ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"]
        .iter()
        .map(|method| DataTableFilterOptionDto {
            label: method.to_string(),
            value: method.to_string(),
        })
        .collect()
}
