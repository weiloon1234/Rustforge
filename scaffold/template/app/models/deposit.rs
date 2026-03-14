#[rf_db_enum(storage = "i16")]
pub enum OwnerType {
    User = 1,
}

#[rf_db_enum(storage = "i16")]
pub enum DepositMethod {
    Manual = 1,
}

#[rf_db_enum(storage = "i16")]
pub enum DepositStatus {
    Pending = 1,
    Approved = 2,
    Rejected = 3,
}

#[rf_db_enum(storage = "i16")]
pub enum DepositReviewAction {
    Approve = 1,
    Reject = 2,
}

#[rf_model(table = "deposits")]
pub struct Deposit {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub owner_type: OwnerType,
    pub owner_id: i64,
    pub admin_id: Option<i64>,
    pub credit_type: CreditType,
    pub deposit_method: DepositMethod,
    pub company_bank_account_id: Option<i64>,
    pub company_crypto_account_id: Option<i64>,
    pub conversion_rate: Option<rust_decimal::Decimal>,
    pub status: DepositStatus,
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
    #[rf(foreign_key = "company_bank_account_id")]
    pub company_bank_account: BelongsTo<CompanyBankAccount>,
    #[rf(foreign_key = "company_crypto_account_id")]
    pub company_crypto_account: BelongsTo<CompanyCryptoAccount>,
}

#[rf_record_impl]
impl DepositRecord {
    pub fn status_label(&self) -> String {
        self.status.explained_label().to_string()
    }
}
