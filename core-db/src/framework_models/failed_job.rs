#[rf_model(table = "failed_jobs")]
pub struct FailedJob {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub job_name: String,
    pub queue: String,
    pub payload: serde_json::Value,
    pub error: String,
    pub attempts: i32,
    pub group_id: Option<String>,
    pub failed_at: time::OffsetDateTime,
}
