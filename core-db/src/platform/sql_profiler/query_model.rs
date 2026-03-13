#[rf_model(table = "sql_profiler_queries", observe = false, profile = false)]
pub struct SqlProfilerQuery {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub request_id: uuid::Uuid,
    pub table_name: String,
    pub operation: String,
    pub sql: String,
    pub binds: String,
    pub duration_us: i64,
    pub created_at: time::OffsetDateTime,
}
