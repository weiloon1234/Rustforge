#[rf_db_enum(storage = "string")]
pub enum CountryStatus {
    Enabled,
    Disabled,
}

#[rf_db_enum(storage = "i16")]
pub enum CountryIsDefault {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "countries")]
pub struct Country {
    #[rf(pk(strategy = manual))]
    pub iso2: String,
    pub iso3: String,
    pub iso_numeric: Option<String>,
    pub name: String,
    pub official_name: Option<String>,
    pub capital: Option<String>,
    pub capitals: Vec<String>,
    pub region: Option<String>,
    pub subregion: Option<String>,
    pub currencies: serde_json::Value,
    pub primary_currency_code: Option<String>,
    pub calling_code: Option<String>,
    pub calling_root: Option<String>,
    pub calling_suffixes: Vec<String>,
    pub tlds: Vec<String>,
    pub timezones: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub independent: Option<bool>,
    pub status: CountryStatus,
    pub conversion_rate: rust_decimal::Decimal,
    pub is_default: CountryIsDefault,
    pub assignment_status: Option<String>,
    pub un_member: Option<bool>,
    pub flag_emoji: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
