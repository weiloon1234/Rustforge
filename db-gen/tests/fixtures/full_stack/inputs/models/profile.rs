#[rf_model(table = "profiles")]
pub struct Profile {
    pub id: i64,
    pub display_name: Localized<String>,
}
