#[rf_model(table = "admins")]
pub struct Admin {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
}
