use core_web::ids::SnowflakeId;
use generated::models::CompanyCryptoAccountStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCompanyCryptoAccountInput {
    pub crypto_network_id: SnowflakeId,
    pub wallet_address: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub conversion_rate: rust_decimal::Decimal,
    pub status: CompanyCryptoAccountStatus,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CompanyCryptoAccountOutput {
    pub id: SnowflakeId,
    pub crypto_network_id: SnowflakeId,
    pub crypto_network_name: Option<String>,
    pub wallet_address: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub conversion_rate: rust_decimal::Decimal,
    pub status: CompanyCryptoAccountStatus,
    pub sort_order: i32,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}
