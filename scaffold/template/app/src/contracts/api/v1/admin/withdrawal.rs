use core_web::ids::SnowflakeId;
use generated::models::{
    CreditType, OwnerType, WithdrawalMethod, WithdrawalReviewAction, WithdrawalStatus,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminWithdrawalReviewInput {
    pub action: WithdrawalReviewAction,
    #[serde(default)]
    pub admin_remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct WithdrawalOutput {
    pub id: SnowflakeId,
    pub owner_type: OwnerType,
    pub owner_id: SnowflakeId,
    pub credit_type: CreditType,
    pub withdrawal_method: WithdrawalMethod,
    pub bank_id: Option<SnowflakeId>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub crypto_network_id: Option<SnowflakeId>,
    pub crypto_wallet_address: Option<String>,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub conversion_rate: Option<rust_decimal::Decimal>,
    pub status: WithdrawalStatus,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub amount: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub fee: rust_decimal::Decimal,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub net_amount: rust_decimal::Decimal,
    pub related_key: Option<String>,
    pub remark: Option<String>,
    pub admin_remark: Option<String>,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub reviewed_at: Option<time::OffsetDateTime>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
}
