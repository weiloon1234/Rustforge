use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::CompanyCryptoAccountStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.company_crypto_account";
pub const ROUTE_PREFIX: &str = "/datatable/company_crypto_account";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CompanyCryptoAccountDatatableRow {
    pub id: SnowflakeId,
    pub crypto_network_id: SnowflakeId,
    pub crypto_network_name: Option<String>,
    pub wallet_address: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub conversion_rate: rust_decimal::Decimal,
    pub status: CompanyCryptoAccountStatus,
    pub status_label: String,
    pub sort_order: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminCompanyCryptoAccountDataTableContract;

impl DataTableScopedContract for AdminCompanyCryptoAccountDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = CompanyCryptoAccountDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Company Crypto Account DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search wallet address".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "crypto_network_id".to_string(),
                    filter_key: "f-crypto_network_id".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Crypto Network ID".to_string(),
                    placeholder: Some("Network ID".to_string()),
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
                    options: Some(CompanyCryptoAccountStatus::datatable_filter_options()),
                },
            ],
        ]
    }
}
