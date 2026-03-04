use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.account";
pub const ROUTE_PREFIX: &str = "/datatable/admin";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableRow {
    pub id: SnowflakeId,
    pub identity: String,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableSummaryOutput {
    #[ts(type = "number")]
    pub total_admin_counts: i64,
    #[ts(type = "number")]
    pub total_filtered: i64,
    #[ts(type = "number")]
    pub developer_count: i64,
    #[ts(type = "number")]
    pub superadmin_count: i64,
    #[ts(type = "number")]
    pub admin_count: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AdminAdminDataTableContract;

impl DataTableScopedContract for AdminAdminDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = AdminDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Account"
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
            vec![DataTableFilterFieldDto {
                field: "username".to_string(),
                filter_key: "f-like-username".to_string(),
                field_type: DataTableFilterFieldType::Text,
                label: "Username".to_string(),
                placeholder: Some("Contains".to_string()),
                description: None,
                options: None,
            }],
            vec![DataTableFilterFieldDto {
                field: "admin_type".to_string(),
                filter_key: "f-admin_type".to_string(),
                field_type: DataTableFilterFieldType::Select,
                label: "Admin Type".to_string(),
                placeholder: Some("Choose type".to_string()),
                description: None,
                options: Some(AdminType::datatable_filter_options()),
            }],
        ]
    }
}
