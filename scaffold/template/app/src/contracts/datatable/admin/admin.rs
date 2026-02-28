use core_datatable::DataTableInput;
use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest,
    DataTableScopedContract,
};
use generated::models::AdminType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableRow {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    #[ts(type = "string[]")]
    pub abilities: Vec<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminAdminDataTableContract;

impl DataTableScopedContract for AdminAdminDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = AdminDatatableRow;

    fn scoped_key(&self) -> &'static str {
        "admin.account"
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Account"
    }

    fn email_to_input(&self, req: &Self::EmailRequest) -> DataTableInput {
        req.to_input()
    }

    fn email_recipients(&self, req: &Self::EmailRequest) -> Vec<String> {
        req.base.recipients.clone()
    }

    fn email_subject(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.subject.clone()
    }

    fn export_file_name(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.export_file_name.clone()
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
