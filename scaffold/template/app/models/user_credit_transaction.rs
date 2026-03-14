use core_i18n::t_args;

#[rf_db_enum(storage = "i16")]
pub enum CreditType {
    Credit1 = 1,
    Credit2 = 2,
}

#[rf_db_enum(storage = "i16")]
pub enum CreditTransactionType {
    AdminAdd = 101,
    AdminDeduct = 102,
    TransferIn = 201,
    TransferOut = 202,
    Withdraw = 301,
    WithdrawRefund = 302,
    TopUp = 401,
}

#[rf_db_enum(storage = "i16")]
pub enum AdjustableCreditType {
    Credit1 = 1,
}

#[rf_model(table = "user_credit_transactions")]
pub struct UserCreditTransaction {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub user_id: i64,
    pub admin_id: Option<i64>,
    pub credit_type: CreditType,
    pub amount: rust_decimal::Decimal,
    pub transaction_type: CreditTransactionType,
    pub related_key: Option<String>,
    pub params: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub custom_description: bool,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub custom_description_text: Localized<String>,
    #[rf(foreign_key = "user_id")]
    pub user: BelongsTo<User>,
    #[rf(foreign_key = "admin_id")]
    pub admin: BelongsTo<Admin>,
}

#[rf_record_impl]
impl UserCreditTransactionRecord {
    pub fn enrich_transaction_type_explained(&mut self) {
        if self.custom_description {
            if let Some(ref text) = self.custom_description_text {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    self.transaction_type_explained = trimmed.to_string();
                    return;
                }
            }
        }

        if let Some(serde_json::Value::Object(ref map)) = self.params {
            if !map.is_empty() {
                let args: Vec<(&str, String)> = map
                    .iter()
                    .map(|(k, v)| {
                        let s = match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k.as_str(), s)
                    })
                    .collect();
                let refs: Vec<(&str, &str)> =
                    args.iter().map(|(k, v)| (*k, v.as_str())).collect();
                self.transaction_type_explained =
                    t_args(self.transaction_type.i18n_key(), &refs);
            }
        }
    }
}
