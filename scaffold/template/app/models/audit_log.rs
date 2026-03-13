#[rf_db_enum(storage = "i16")]
pub enum AuditAction {
    Create = 1,
    Update = 2,
    Delete = 3,
}

#[rf_model(table = "audit_logs", observe = false)]
pub struct AuditLog {
    #[rf(pk(strategy = snowflake))]
    pub id: i64,
    pub admin_id: i64,
    pub action: AuditAction,
    pub table_name: String,
    pub record_key: String,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub created_at: time::OffsetDateTime,
    #[rf(foreign_key = "admin_id")]
    pub admin: BelongsTo<Admin>,
}
