use core_web::ids::SnowflakeId;
use core_web::{DateTime, Decimal};
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
    pub conversion_rate: Decimal,
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
    pub conversion_rate: Decimal,
    pub status: CompanyCryptoAccountStatus,
    pub sort_order: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
