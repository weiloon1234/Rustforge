use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::BankStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.bank";
pub const ROUTE_PREFIX: &str = "/datatable/bank";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct BankDatatableRow {
    pub id: SnowflakeId,
    pub country_iso2: String,
    pub name: String,
    pub code: Option<String>,
    pub status: BankStatus,
    pub status_label: String,
    pub sort_order: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminBankDataTableContract;

impl DataTableScopedContract for AdminBankDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = BankDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Bank DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search name/code".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "country_iso2".to_string(),
                    filter_key: "f-country_iso2".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Country".to_string(),
                    placeholder: Some("Country ISO2".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "status".to_string(),
                    filter_key: "f-status".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Status".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(BankStatus::datatable_filter_options()),
                },
            ],
        ]
    }
}
