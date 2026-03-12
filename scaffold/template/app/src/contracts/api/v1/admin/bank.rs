use core_web::ids::SnowflakeId;
use generated::models::BankStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminBankInput {
    pub country_iso2: String,
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    pub status: BankStatus,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct BankOutput {
    pub id: SnowflakeId,
    pub country_iso2: String,
    pub name: String,
    pub code: Option<String>,
    pub logo_url: Option<String>,
    pub status: BankStatus,
    pub sort_order: i32,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}
