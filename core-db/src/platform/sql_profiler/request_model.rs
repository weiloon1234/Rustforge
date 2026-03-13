#[rf_model(table = "sql_profiler_requests", observe = false, profile = false)]
pub struct SqlProfilerRequest {
    #[rf(pk(strategy = manual))]
    pub id: uuid::Uuid,
    pub request_method: String,
    pub request_path: String,
    pub total_queries: i32,
    pub total_duration_ms: f64,
    pub created_at: time::OffsetDateTime,
}
