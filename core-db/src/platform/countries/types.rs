#![allow(dead_code)]

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
pub enum PhoneNumberOutputFormat {
    E164,
    International,
    National,
    Rfc3966,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, TS)]
#[serde(rename_all = "snake_case")]
pub enum CountryStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountryStatusFilterOption {
    pub label: &'static str,
    pub value: &'static str,
}

impl CountryStatus {
    pub const ALL: [Self; 2] = [Self::Enabled, Self::Disabled];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub fn datatable_filter_options() -> Vec<CountryStatusFilterOption> {
        Self::ALL
            .iter()
            .map(|status| CountryStatusFilterOption {
                label: status.label(),
                value: status.as_str(),
            })
            .collect()
    }
}

pub const COUNTRY_STATUS_ENABLED: &str = CountryStatus::Enabled.as_str();
pub const COUNTRY_STATUS_DISABLED: &str = CountryStatus::Disabled.as_str();
pub const COUNTRY_ISO2_LEN: usize = 2;
const BUILTIN_COUNTRIES_JSON: &str = include_str!("seed/countries.seed.json");

pub fn normalize_country_status(value: &str) -> Option<&'static str> {
    CountryStatus::from_str(value).map(CountryStatus::as_str)
}

pub fn normalize_country_iso2(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.len() != COUNTRY_ISO2_LEN {
        return None;
    }
    if !normalized.chars().all(|ch| ch.is_ascii_uppercase()) {
        return None;
    }
    Some(normalized)
}

pub fn load_builtin_country_seed() -> anyhow::Result<Vec<CountrySeed>> {
    serde_json::from_str(BUILTIN_COUNTRIES_JSON).context("failed to parse built-in countries seed")
}

pub fn normalize_country_seed(mut seed: CountrySeed) -> CountrySeed {
    seed.iso2 = seed.iso2.trim().to_ascii_uppercase();
    seed.iso3 = seed.iso3.trim().to_ascii_uppercase();
    seed.iso_numeric = seed.iso_numeric.and_then(trim_opt);
    seed.name = seed.name.trim().to_string();
    seed.official_name = seed.official_name.and_then(trim_opt);
    seed.capital = seed.capital.and_then(trim_opt);
    seed.capitals = normalize_vec(seed.capitals);
    seed.region = seed.region.and_then(trim_opt);
    seed.subregion = seed.subregion.and_then(trim_opt);
    seed.primary_currency_code = seed
        .primary_currency_code
        .and_then(trim_opt)
        .map(|v| v.to_ascii_uppercase());
    seed.calling_code = seed.calling_code.and_then(trim_opt);
    seed.calling_root = seed.calling_root.and_then(trim_opt);
    seed.calling_suffixes = normalize_vec(seed.calling_suffixes);
    seed.tlds = normalize_vec(seed.tlds);
    seed.timezones = normalize_vec(seed.timezones);
    seed.assignment_status = seed.assignment_status.and_then(trim_opt);
    seed.flag_emoji = seed.flag_emoji.and_then(trim_opt);

    seed.currencies = seed
        .currencies
        .into_iter()
        .map(|mut c| {
            c.code = c.code.trim().to_ascii_uppercase();
            c.name = c.name.and_then(trim_opt);
            c.symbol = c.symbol.and_then(trim_opt);
            c
        })
        .filter(|c| !c.code.is_empty())
        .collect();

    seed
}

pub fn default_country_status_for_iso2(iso2: &str) -> &'static str {
    if iso2.eq_ignore_ascii_case("MY") {
        COUNTRY_STATUS_ENABLED
    } else {
        COUNTRY_STATUS_DISABLED
    }
}

fn trim_opt(input: String) -> Option<String> {
    let value = input.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn normalize_vec(input: Vec<String>) -> Vec<String> {
    input.into_iter().filter_map(trim_opt).collect::<Vec<_>>()
}

#[derive(Debug, thiserror::Error)]
pub enum PhoneNumberFormatError {
    #[error("invalid phone number country: {iso2}")]
    InvalidCountry { iso2: String },
    #[error("invalid phone number for country {iso2}: {input}")]
    InvalidNumber { iso2: String, input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct CountryCurrency {
    pub code: String,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub symbol: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub minor_units: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CountrySeed {
    pub iso2: String,
    pub iso3: String,
    #[serde(default)]
    pub iso_numeric: Option<String>,
    pub name: String,
    #[serde(default)]
    pub official_name: Option<String>,
    #[serde(default)]
    pub capital: Option<String>,
    #[serde(default)]
    pub capitals: Vec<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub subregion: Option<String>,
    #[serde(default)]
    pub currencies: Vec<CountryCurrency>,
    #[serde(default)]
    pub primary_currency_code: Option<String>,
    #[serde(default)]
    pub calling_code: Option<String>,
    #[serde(default)]
    pub calling_root: Option<String>,
    #[serde(default)]
    pub calling_suffixes: Vec<String>,
    #[serde(default)]
    pub tlds: Vec<String>,
    #[serde(default)]
    pub timezones: Vec<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub independent: Option<bool>,
    #[serde(default, alias = "status")]
    pub assignment_status: Option<String>,
    #[serde(default)]
    pub un_member: Option<bool>,
    #[serde(default)]
    pub flag_emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(rename = "CountryRuntime")]
pub struct Country {
    pub iso2: String,
    pub iso3: String,
    #[ts(optional)]
    pub iso_numeric: Option<String>,
    pub name: String,
    #[ts(optional)]
    pub official_name: Option<String>,
    #[ts(optional)]
    pub capital: Option<String>,
    pub capitals: Vec<String>,
    #[ts(optional)]
    pub region: Option<String>,
    #[ts(optional)]
    pub subregion: Option<String>,
    pub currencies: Vec<CountryCurrency>,
    #[ts(optional)]
    pub primary_currency_code: Option<String>,
    #[ts(optional)]
    pub calling_code: Option<String>,
    #[ts(optional)]
    pub calling_root: Option<String>,
    pub calling_suffixes: Vec<String>,
    pub tlds: Vec<String>,
    pub timezones: Vec<String>,
    #[ts(optional)]
    pub latitude: Option<f64>,
    #[ts(optional)]
    pub longitude: Option<f64>,
    #[ts(optional)]
    pub independent: Option<bool>,
    pub status: String,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub conversion_rate: rust_decimal::Decimal,
    pub is_default: bool,
    #[ts(optional)]
    pub assignment_status: Option<String>,
    #[ts(optional)]
    pub un_member: Option<bool>,
    #[ts(optional)]
    pub flag_emoji: Option<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

impl Country {
    /// Format a phone number into canonical E.164 by default.
    ///
    /// If `throw_if_invalid` is false, invalid input returns `Ok(None)`.
    pub fn format_phone_number(
        &self,
        raw: &str,
        throw_if_invalid: bool,
    ) -> Result<Option<String>, PhoneNumberFormatError> {
        self.format_phone_number_as(raw, PhoneNumberOutputFormat::E164, throw_if_invalid)
    }

    /// Format a phone number to a specific output format.
    ///
    /// If `throw_if_invalid` is false, invalid input returns `Ok(None)`.
    pub fn format_phone_number_as(
        &self,
        raw: &str,
        format: PhoneNumberOutputFormat,
        throw_if_invalid: bool,
    ) -> Result<Option<String>, PhoneNumberFormatError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::invalid_or_none(
                throw_if_invalid,
                PhoneNumberFormatError::InvalidNumber {
                    iso2: self.iso2.clone(),
                    input: raw.to_string(),
                },
            );
        }

        let region = match self.iso2.parse::<phonenumber::country::Id>() {
            Ok(value) => value,
            Err(_) => {
                return Self::invalid_or_none(
                    throw_if_invalid,
                    PhoneNumberFormatError::InvalidCountry {
                        iso2: self.iso2.clone(),
                    },
                );
            }
        };

        let parsed = match phonenumber::parse(Some(region), trimmed) {
            Ok(value) => value,
            Err(_) => {
                return Self::invalid_or_none(
                    throw_if_invalid,
                    PhoneNumberFormatError::InvalidNumber {
                        iso2: self.iso2.clone(),
                        input: raw.to_string(),
                    },
                );
            }
        };

        if !phonenumber::is_valid(&parsed) {
            return Self::invalid_or_none(
                throw_if_invalid,
                PhoneNumberFormatError::InvalidNumber {
                    iso2: self.iso2.clone(),
                    input: raw.to_string(),
                },
            );
        }

        let mode = match format {
            PhoneNumberOutputFormat::E164 => phonenumber::Mode::E164,
            PhoneNumberOutputFormat::International => phonenumber::Mode::International,
            PhoneNumberOutputFormat::National => phonenumber::Mode::National,
            PhoneNumberOutputFormat::Rfc3966 => phonenumber::Mode::Rfc3966,
        };

        Ok(Some(parsed.format().mode(mode).to_string()))
    }

    fn invalid_or_none(
        throw_if_invalid: bool,
        err: PhoneNumberFormatError,
    ) -> Result<Option<String>, PhoneNumberFormatError> {
        if throw_if_invalid {
            Err(err)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_country_iso2;

    #[test]
    fn normalize_country_iso2_accepts_valid_value() {
        assert_eq!(normalize_country_iso2("my"), Some("MY".to_string()));
        assert_eq!(normalize_country_iso2(" US "), Some("US".to_string()));
    }

    #[test]
    fn normalize_country_iso2_rejects_invalid_values() {
        assert_eq!(normalize_country_iso2(""), None);
        assert_eq!(normalize_country_iso2("M"), None);
        assert_eq!(normalize_country_iso2("MY1"), None);
        assert_eq!(normalize_country_iso2("1Y"), None);
        assert_eq!(normalize_country_iso2("🇲🇾"), None);
    }
}
