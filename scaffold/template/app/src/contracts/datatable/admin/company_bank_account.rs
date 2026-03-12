use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::CompanyBankAccountStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.company_bank_account";
pub const ROUTE_PREFIX: &str = "/datatable/company_bank_account";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CompanyBankAccountDatatableRow {
    pub id: SnowflakeId,
    pub bank_id: SnowflakeId,
    pub bank_name: Option<String>,
    pub account_name: String,
    pub account_number: String,
    pub status: CompanyBankAccountStatus,
    pub status_label: String,
    pub sort_order: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminCompanyBankAccountDataTableContract;

impl DataTableScopedContract for AdminCompanyBankAccountDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = CompanyBankAccountDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Company Bank Account DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search account name/number".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "bank_id".to_string(),
                    filter_key: "f-bank_id".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Bank ID".to_string(),
                    placeholder: Some("Bank ID".to_string()),
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
                    options: Some(CompanyBankAccountStatus::datatable_filter_options()),
                },
            ],
        ]
    }
}
