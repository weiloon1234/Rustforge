use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::CryptoNetworkStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.crypto_network";
pub const ROUTE_PREFIX: &str = "/datatable/crypto_network";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CryptoNetworkDatatableRow {
    pub id: SnowflakeId,
    pub name: String,
    pub symbol: String,
    pub status: CryptoNetworkStatus,
    pub status_label: String,
    pub sort_order: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminCryptoNetworkDataTableContract;

impl DataTableScopedContract for AdminCryptoNetworkDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = CryptoNetworkDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Crypto Network DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search name/symbol".to_string()),
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
                    options: Some(CryptoNetworkStatus::datatable_filter_options()),
                },
            ],
        ]
    }
}
