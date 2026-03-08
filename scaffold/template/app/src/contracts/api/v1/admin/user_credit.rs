use core_web::ids::SnowflakeId;
use generated::models::{CreditTransactionType, CreditType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCreditAdjustInput {
    pub username: String,
    pub credit_type: CreditType,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UserCreditTransactionOutput {
    pub id: SnowflakeId,
    pub user_id: SnowflakeId,
    pub credit_type: CreditType,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    pub transaction_type: CreditTransactionType,
    pub related_key: Option<String>,
    pub remark: Option<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
}
