#[rf_model(table = "meta")]
pub struct Meta {
    #[rf(pk(strategy = manual))]
    pub id: i64,
    pub owner_type: String,
    pub owner_id: i64,
    pub field: String,
    pub value: serde_json::Value,
}
