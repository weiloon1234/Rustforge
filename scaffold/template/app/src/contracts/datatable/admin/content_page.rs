use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use generated::models::PageSystemFlag;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.content_page";
pub const ROUTE_PREFIX: &str = "/datatable/content_page";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct PageDatatableRow {
    pub id: i64,
    pub tag: String,
    pub title: Option<String>,
    #[ts(type = "PageSystemFlag")]
    pub is_system: PageSystemFlag,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminPageDataTableContract;

impl DataTableScopedContract for AdminPageDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = PageDatatableRow;

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
                options: Some(PageSystemFlag::datatable_filter_options()),
            },
        ]]
    }
}
