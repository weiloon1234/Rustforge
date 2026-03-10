use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableGenericEmailExportRequest,
    DataTableGenericQueryRequest, DataTableScopedContract,
};
use core_web::ids::SnowflakeId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.introducer_change";
pub const ROUTE_PREFIX: &str = "/datatable/introducer_change";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct IntroducerChangeDatatableRow {
    pub id: SnowflakeId,
    pub user_id: SnowflakeId,
    pub from_user_id: Option<SnowflakeId>,
    pub to_user_id: SnowflakeId,
    pub admin_id: SnowflakeId,
    pub remark: Option<String>,
    pub user_username: Option<String>,
    pub from_username: Option<String>,
    pub to_username: Option<String>,
    pub admin_username: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminIntroducerChangeDataTableContract;

impl DataTableScopedContract for AdminIntroducerChangeDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = IntroducerChangeDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Introducer Change DataTable"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![DataTableFilterFieldDto {
                field: "q".to_string(),
                filter_key: "q".to_string(),
                field_type: DataTableFilterFieldType::Text,
                label: "Keyword".to_string(),
                placeholder: Some("Search username".to_string()),
                description: None,
                options: None,
            }],
            vec![
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
