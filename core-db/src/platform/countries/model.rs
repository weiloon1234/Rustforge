use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow)]
pub struct CountryRow {
    pub iso2: String,
    pub iso3: String,
    pub iso_numeric: Option<String>,
    pub name: String,
    pub official_name: Option<String>,
    pub capital: Option<String>,
    pub capitals: Vec<String>,
    pub region: Option<String>,
    pub subregion: Option<String>,
    pub currencies: Value,
    pub primary_currency_code: Option<String>,
    pub calling_code: Option<String>,
    pub calling_root: Option<String>,
    pub calling_suffixes: Vec<String>,
    pub tlds: Vec<String>,
    pub timezones: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub independent: Option<bool>,
    pub status: String,
    pub assignment_status: Option<String>,
    pub un_member: Option<bool>,
    pub flag_emoji: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
