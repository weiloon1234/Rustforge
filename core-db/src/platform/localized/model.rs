#[rf_model(table = "localized")]
pub struct Localized {
    #[rf(pk(strategy = manual))]
    pub id: i64,
    pub owner_type: String,
    pub owner_id: i64,
    pub field: String,
    pub locale: String,
    pub value: String,
}
