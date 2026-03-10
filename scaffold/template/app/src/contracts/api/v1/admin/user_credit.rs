use core_web::ids::SnowflakeId;
use generated::localized::LocalizedInput;
use generated::models::{AdjustableCreditType, CreditTransactionType, CreditType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminCreditAdjustInput {
    pub username: String,
    pub credit_type: AdjustableCreditType,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    #[serde(default)]
    pub remark: Option<String>,
    /// Localized custom description
    #[serde(default)]
    pub custom_description: Option<LocalizedInput>,
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
