#[rf_model(table = "webhook_logs")]
pub struct WebhookLog {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: time::OffsetDateTime,
}
