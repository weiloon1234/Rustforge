#![allow(dead_code)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhoneNumberOutputFormat {
    E164,
    International,
    National,
    Rfc3966,
}

#[derive(Debug, thiserror::Error)]
pub enum PhoneNumberFormatError {
    #[error("invalid phone number country: {iso2}")]
    InvalidCountry { iso2: String },
    #[error("invalid phone number for country {iso2}: {input}")]
    InvalidNumber { iso2: String, input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CountryCurrency {
    pub code: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Country {
    pub iso2: String,
    pub iso3: String,
    pub iso_numeric: Option<String>,
    pub name: String,
    pub official_name: Option<String>,
    pub capital: Option<String>,
    pub capitals: Vec<String>,
    pub region: Option<String>,
    pub subregion: Option<String>,
    pub currencies: Vec<CountryCurrency>,
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
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
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
