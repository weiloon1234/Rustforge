use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableFilterOptionDto,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest, DataTableScopedContract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.http_client_log";
pub const ROUTE_PREFIX: &str = "/datatable/http-client-log";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct HttpClientLogDatatableRow {
    pub id: String,
    pub request_url: String,
    pub request_method: String,
    pub response_status: Option<i32>,
    pub duration_ms: Option<i32>,
    #[ts(type = "Record<string, unknown> | null")]
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    #[ts(type = "Record<string, unknown> | null")]
    pub response_headers: Option<serde_json::Value>,
    pub response_body: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminHttpClientLogDataTableContract;

impl DataTableScopedContract for AdminHttpClientLogDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = HttpClientLogDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin HTTP Client Log"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "request_url".to_string(),
                    filter_key: "f-like-request_url".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "URL".to_string(),
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
                    field: "response_status".to_string(),
                    filter_key: "f-response_status".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Status Code".to_string(),
                    placeholder: Some("e.g. 200".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "duration_ms_min".to_string(),
                    filter_key: "f-gte-duration_ms".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Min Duration (ms)".to_string(),
                    placeholder: Some("e.g. 50".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "duration_ms_max".to_string(),
                    filter_key: "f-lte-duration_ms".to_string(),
                    field_type: DataTableFilterFieldType::Number,
                    label: "Max Duration (ms)".to_string(),
                    placeholder: Some("e.g. 3000".to_string()),
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
