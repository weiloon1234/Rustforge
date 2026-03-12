use core_web::ids::SnowflakeId;
use generated::models::CryptoNetworkStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCryptoNetworkInput {
    pub name: String,
    pub symbol: String,
    pub status: CryptoNetworkStatus,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CryptoNetworkOutput {
    pub id: SnowflakeId,
    pub name: String,
    pub symbol: String,
    pub logo_url: Option<String>,
    pub status: CryptoNetworkStatus,
    pub sort_order: i32,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}
