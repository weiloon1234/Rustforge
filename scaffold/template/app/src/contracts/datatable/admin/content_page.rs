use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.content_page";
pub const ROUTE_PREFIX: &str = "/datatable/content_page";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ContentPageDatatableRow {
    pub id: String,
    pub tag: String,
    pub title: Option<String>,
    pub is_system: generated::models::ContentPageSystemFlag,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminContentPageDataTableContract;

impl DataTableScopedContract for AdminContentPageDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = ContentPageDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Page"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![vec![
            DataTableFilterFieldDto {
                field: "tag".to_string(),
                filter_key: "f-like-tag".to_string(),
                field_type: DataTableFilterFieldType::Text,
                label: "Tag".to_string(),
                placeholder: Some("Contains".to_string()),
                description: None,
                options: None,
            },
            DataTableFilterFieldDto {
                field: "is_system".to_string(),
                filter_key: "f-is_system".to_string(),
                field_type: DataTableFilterFieldType::Select,
                label: "System".to_string(),
                placeholder: Some("All".to_string()),
                description: None,
                options: Some(generated::models::ContentPageSystemFlag::datatable_filter_options()),
            },
        ]]
    }
}
