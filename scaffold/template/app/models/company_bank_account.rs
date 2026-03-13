#[rf_db_enum(storage = "i16")]
pub enum CompanyBankAccountStatus {
    Enabled = 1,
    Disabled = 2,
}

#[rf_model(table = "company_bank_accounts")]
pub struct CompanyBankAccount {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub bank_id: i64,
    pub account_name: String,
    pub account_number: String,
    pub status: CompanyBankAccountStatus,
    pub sort_order: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "bank_id")]
    pub bank: BelongsTo<Bank>,
}
