use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::{CreditTransactionType, CreditType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.user_credit_transaction";
pub const ROUTE_PREFIX: &str = "/datatable/user_credit_transaction";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserCreditTransactionDatatableRow {
    pub id: SnowflakeId,
    pub user_id: SnowflakeId,
    pub admin_id: Option<SnowflakeId>,
    pub credit_type: CreditType,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    pub transaction_type: CreditTransactionType,
    /// Backend-computed human-readable label for the transaction type,
    /// respecting current locale and interpolating dynamic `params`.
    /// If `remark` is set, it overrides this value.
    pub transaction_type_explained: String,
    pub related_key: Option<String>,
    pub remark: Option<String>,
    pub user_username: Option<String>,
    pub admin_username: Option<String>,
    pub custom_description: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminUserCreditTransactionDataTableContract;

impl DataTableScopedContract for AdminUserCreditTransactionDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = UserCreditTransactionDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin User Credit Transaction DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search username".to_string()),
                    description: None,
                    options: None,
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
                    field: "transaction_type".to_string(),
                    filter_key: "f-transaction_type".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Transaction Type".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(CreditTransactionType::datatable_filter_options()),
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
