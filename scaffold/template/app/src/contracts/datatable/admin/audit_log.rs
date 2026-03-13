use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableFilterOptionDto,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use generated::models::AuditAction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.audit_log";
pub const ROUTE_PREFIX: &str = "/datatable/audit_log";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AuditLogDatatableRow {
    pub id: SnowflakeId,
    #[ts(type = "string")]
    pub admin_id: i64,
    pub admin_username: String,
    pub action: AuditAction,
    pub action_explained: String,
    pub table_name: String,
    pub record_key: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub old_data: Option<serde_json::Value>,
    #[ts(type = "Record<string, unknown> | null")]
    pub new_data: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminAuditLogDataTableContract;

impl DataTableScopedContract for AdminAuditLogDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = AuditLogDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Audit Log"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search old/new data JSON".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "table_name".to_string(),
                    filter_key: "f-table_name".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Model".to_string(),
                    placeholder: Some("All models".to_string()),
                    description: None,
                    options: Some(vec![
                        DataTableFilterOptionDto {
                            value: "admin".to_string(),
                            label: "Admin".to_string(),
                        },
                        DataTableFilterOptionDto {
                            value: "users".to_string(),
                            label: "Users".to_string(),
                        },
                        DataTableFilterOptionDto {
                            value: "countries".to_string(),
                            label: "Countries".to_string(),
                        },
                        DataTableFilterOptionDto {
                            value: "content_pages".to_string(),
                            label: "Content Pages".to_string(),
                        },
                        DataTableFilterOptionDto {
                            value: "user_credit_transactions".to_string(),
                            label: "Credit Transactions".to_string(),
                        },
                        DataTableFilterOptionDto {
                            value: "introducer_changes".to_string(),
                            label: "Introducer Changes".to_string(),
                        },
                    ]),
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "action".to_string(),
                    filter_key: "f-action".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Action".to_string(),
                    placeholder: Some("All actions".to_string()),
                    description: None,
                    options: Some(AuditAction::datatable_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "record_key".to_string(),
                    filter_key: "f-record_key".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Record Key".to_string(),
                    placeholder: Some("Exact match".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![
                DataTableFilterFieldDto {
                    field: "created_at".to_string(),
                    filter_key: "f-date-from-created_at".to_string(),
                    field_type: DataTableFilterFieldType::Date,
                    label: "From".to_string(),
                    placeholder: None,
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "created_at".to_string(),
                    filter_key: "f-date-to-created_at".to_string(),
                    field_type: DataTableFilterFieldType::Date,
                    label: "To".to_string(),
                    placeholder: None,
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}
