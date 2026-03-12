use core_web::ids::SnowflakeId;
use generated::models::CompanyBankAccountStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCompanyBankAccountInput {
    pub bank_id: SnowflakeId,
    pub account_name: String,
    pub account_number: String,
    pub status: CompanyBankAccountStatus,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CompanyBankAccountOutput {
    pub id: SnowflakeId,
    pub bank_id: SnowflakeId,
    pub bank_name: Option<String>,
    pub account_name: String,
    pub account_number: String,
    pub status: CompanyBankAccountStatus,
    pub sort_order: i32,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}
