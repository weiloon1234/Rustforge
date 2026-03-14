#[rf_db_enum(storage = "i16")]
pub enum WithdrawalMethod {
    Manual = 1,
}

#[rf_db_enum(storage = "i16")]
pub enum WithdrawalStatus {
    Pending = 1,
    Processing = 2,
    Approved = 3,
    Rejected = 4,
}

#[rf_db_enum(storage = "i16")]
pub enum WithdrawalReviewAction {
    Process = 1,
    Approve = 2,
    Reject = 3,
}

#[rf_model(table = "withdrawals")]
pub struct Withdrawal {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub owner_type: OwnerType,
    pub owner_id: i64,
    pub admin_id: Option<i64>,
    pub credit_type: CreditType,
    pub withdrawal_method: WithdrawalMethod,
    pub bank_id: Option<i64>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub crypto_network_id: Option<i64>,
    pub crypto_wallet_address: Option<String>,
    pub conversion_rate: Option<rust_decimal::Decimal>,
    pub status: WithdrawalStatus,
    pub amount: rust_decimal::Decimal,
    pub fee: rust_decimal::Decimal,
    pub net_amount: rust_decimal::Decimal,
    pub related_key: Option<String>,
    pub params: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub admin_remark: Option<String>,
    pub reviewed_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "admin_id")]
    pub admin: BelongsTo<Admin>,
    #[rf(foreign_key = "bank_id")]
    pub bank: BelongsTo<Bank>,
    #[rf(foreign_key = "crypto_network_id")]
    pub crypto_network: BelongsTo<CryptoNetwork>,
}

#[rf_record_impl]
impl WithdrawalRecord {
    pub fn status_label(&self) -> String {
        self.status.explained_label().to_string()
    }
}
