use core_web::datatable::{
    DataTableFilterFieldDto, DataTableFilterFieldType, DataTableFilterOptionDto,
    DataTableGenericEmailExportRequest, DataTableGenericQueryRequest, DataTableScopedContract,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SCOPED_KEY: &str = "admin.country";
pub const ROUTE_PREFIX: &str = "/datatable/country";
const COUNTRY_STATUS_ENABLED: &str = "enabled";
const COUNTRY_STATUS_DISABLED: &str = "disabled";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CountryDatatableRow {
    pub id: String,
    pub iso2: String,
    pub iso3: String,
    pub name: String,
    pub region: Option<String>,
    pub calling_code: Option<String>,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct AdminCountryDataTableContract;

impl DataTableScopedContract for AdminCountryDataTableContract {
    type QueryRequest = DataTableGenericQueryRequest;
    type EmailRequest = DataTableGenericEmailExportRequest;
    type Row = CountryDatatableRow;

    fn scoped_key(&self) -> &'static str {
        SCOPED_KEY
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Country"
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![DataTableFilterFieldDto {
                field: "q".to_string(),
                filter_key: "q".to_string(),
                field_type: DataTableFilterFieldType::Text,
                label: "Keyword".to_string(),
                placeholder: Some("Search name/iso/calling code".to_string()),
                description: None,
                options: None,
            }],
            vec![
                DataTableFilterFieldDto {
                    field: "status".to_string(),
                    filter_key: "status".to_string(),
                    field_type: DataTableFilterFieldType::Select,
                    label: "Status".to_string(),
                    placeholder: Some("All".to_string()),
                    description: None,
                    options: Some(status_filter_options()),
                },
                DataTableFilterFieldDto {
                    field: "region".to_string(),
                    filter_key: "region".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Region".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
        ]
    }
}

fn status_filter_options() -> Vec<DataTableFilterOptionDto> {
    vec![
        DataTableFilterOptionDto {
            label: "Enabled".to_string(),
            value: COUNTRY_STATUS_ENABLED.to_string(),
        },
        DataTableFilterOptionDto {
            label: "Disabled".to_string(),
            value: COUNTRY_STATUS_DISABLED.to_string(),
        },
    ]
}
