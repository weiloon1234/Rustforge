use core_web::ids::SnowflakeId;
use generated::localized::LocalizedInput;
use generated::models::{AdjustableCreditType, CreditTransactionType, CreditType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

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
    /// Enable custom description
    #[serde(default)]
    pub custom_description: bool,
    /// Localized custom description text (required when custom_description is true)
    #[serde(default)]
    pub custom_description_text: Option<LocalizedInput>,
}

impl Validate for AdminCreditAdjustInput {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        if !self.custom_description {
            return Ok(());
        }
        match &self.custom_description_text {
            Some(input) => input.validate(),
            None => {
                let mut errors = validator::ValidationErrors::new();
                errors.add(
                    "custom_description_text",
                    validator::ValidationError::new("required")
                        .with_message(std::borrow::Cow::Borrowed("Custom description text is required.")),
                );
                Err(errors)
            }
        }
    }
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
