use core_web::contracts::rustforge_contract;
use core_web::DateTime;
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCountryStatusUpdateInput {
    #[rf(one_of("enabled", "disabled"))]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCountryStatusUpdateOutput {
    pub iso2: String,
    pub status: String,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCountrySetDefaultOutput {
    pub iso2: String,
    pub is_default: bool,
    pub updated_at: DateTime,
}
