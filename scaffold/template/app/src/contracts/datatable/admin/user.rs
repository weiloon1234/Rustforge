use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::UserBanStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.user";
pub const ROUTE_PREFIX: &str = "/datatable/user";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserDatatableRow {
    pub id: SnowflakeId,
    pub uuid: String,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub country_iso2: Option<String>,
    pub contact_number: Option<String>,
    pub ban: UserBanStatus,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub credit_1: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub credit_2: rust_decimal::Decimal,
    pub introducer_user_id: Option<SnowflakeId>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserDatatableSummaryOutput {
    #[ts(type = "number")]
    pub total_user_count: i64,
    #[ts(type = "number")]
    pub total_filtered: i64,
    #[ts(type = "number")]
    pub banned_count: i64,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub total_credit_1: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub total_credit_2: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Default)]
pub struct AdminUserDataTableContract;

impl DataTableScopedContract for AdminUserDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = UserDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin User DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search name/username/email".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "email".to_string(),
                    filter_key: "f-like-email".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Email".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "username".to_string(),
                    filter_key: "f-like-username".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Username".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "country_iso2".to_string(),
                    filter_key: "f-like-country_iso2".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Country".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "ban".to_string(),
                    filter_key: "f-ban".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Ban Status".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(UserBanStatus::datatable_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "introducer".to_string(),
                    filter_key: "f-like-introducer".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Introducer".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}
