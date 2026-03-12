use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::{CreditType, DepositMethod, DepositStatus, OwnerType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.deposit";
pub const ROUTE_PREFIX: &str = "/datatable/deposit";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct DepositDatatableRow {
    pub id: SnowflakeId,
    pub owner_type: OwnerType,
    pub owner_id: SnowflakeId,
    pub admin_id: Option<SnowflakeId>,
    pub credit_type: CreditType,
    pub deposit_method: DepositMethod,
    pub company_bank_account_id: Option<SnowflakeId>,
    pub company_bank_account_name: Option<String>,
    pub company_crypto_account_id: Option<SnowflakeId>,
    pub company_crypto_network_name: Option<String>,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub conversion_rate: Option<rust_decimal::Decimal>,
    pub status: DepositStatus,
    pub status_label: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub fee: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub net_amount: rust_decimal::Decimal,
    pub related_key: Option<String>,
    pub remark: Option<String>,
    pub admin_remark: Option<String>,
    pub owner_name: Option<String>,
    pub admin_username: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminDepositDataTableContract;

impl DataTableScopedContract for AdminDepositDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = DepositDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Deposit DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search owner username".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "owner_type".to_string(),
                    filter_key: "f-owner_type".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Owner Type".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(OwnerType::datatable_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "credit_type".to_string(),
                    filter_key: "f-credit_type".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Credit Type".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(CreditType::datatable_filter_options()),
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "deposit_method".to_string(),
                    filter_key: "f-deposit_method".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Deposit Method".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(DepositMethod::datatable_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "status".to_string(),
                    filter_key: "f-status".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Status".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(DepositStatus::datatable_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "created_at_from".to_string(),
                    filter_key: "f-date-from-created_at".to_string(),
                    field_type: DataTableFilterFieldType::Date,
                    label: "Created At From".to_string(),
                    placeholder: Some("Start datetime".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "created_at_to".to_string(),
                    filter_key: "f-date-to-created_at".to_string(),
                    field_type: DataTableFilterFieldType::Date,
                    label: "Created At To".to_string(),
                    placeholder: Some("End datetime".to_string()),
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}
