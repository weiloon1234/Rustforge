#[rf_model(table = "outbox_jobs")]
pub struct OutboxJob {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub queue: String,
    pub payload: serde_json::Value,
    pub created_at: time::OffsetDateTime,
}
