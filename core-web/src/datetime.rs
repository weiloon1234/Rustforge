use std::fmt;
use std::ops::Deref;

use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::format_description::well_known::Rfc3339;

/// RFC 3339 datetime newtype that serializes as a string on the wire.
///
/// Wraps `time::OffsetDateTime` with built-in `JsonSchema` and `TS` impls
/// so contract structs need zero per-field annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(pub time::OffsetDateTime);

impl DateTime {
    pub fn now() -> Self {
        Self(time::OffsetDateTime::now_utc())
    }
}

impl Deref for DateTime {
    type Target = time::OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<time::OffsetDateTime> for DateTime {
    fn from(value: time::OffsetDateTime) -> Self {
        Self(value)
    }
}

impl From<DateTime> for time::OffsetDateTime {
    fn from(value: DateTime) -> Self {
        value.0
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .0
            .format(&Rfc3339)
            .map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self
            .0
            .format(&Rfc3339)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an RFC 3339 datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        time::OffsetDateTime::parse(value, &Rfc3339)
            .map(DateTime)
            .map_err(|err| E::custom(format!("invalid RFC 3339 datetime: {err}")))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

impl JsonSchema for DateTime {
    fn schema_name() -> String {
        "DateTime".to_string()
    }

    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <String as JsonSchema>::json_schema(generator)
    }

    fn _schemars_private_non_optional_json_schema(
        generator: &mut schemars::gen::SchemaGenerator,
    ) -> schemars::schema::Schema {
        <String as JsonSchema>::json_schema(generator)
    }

    fn _schemars_private_is_option() -> bool {
        false
    }
}

impl ts_rs::TS for DateTime {
    type WithoutGenerics = Self;

    fn name() -> String {
        <String as ts_rs::TS>::name()
    }

    fn inline() -> String {
        <String as ts_rs::TS>::inline()
    }

    fn inline_flattened() -> String {
        <String as ts_rs::TS>::inline_flattened()
    }

    fn decl() -> String {
        <String as ts_rs::TS>::decl()
    }

    fn decl_concrete() -> String {
        <String as ts_rs::TS>::decl_concrete()
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    use super::DateTime;

    #[test]
    fn serde_round_trip() {
        let now = DateTime::now();
        let json = serde_json::to_string(&now).expect("serialize");
        let back: DateTime = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(now, back);
    }

    #[test]
    fn schema_is_string() {
        let schema = schema_for!(DateTime);
        let text = serde_json::to_string(&schema).expect("schema json");
        assert!(text.contains("\"type\":\"string\""));
    }

    #[test]
    fn ts_output_is_string() {
        assert_eq!(<DateTime as ts_rs::TS>::name(), "string");
    }

    #[test]
    fn display_is_rfc3339() {
        let dt = DateTime::now();
        let display = dt.to_string();
        // Should be parseable back
        let _parsed =
            time::OffsetDateTime::parse(&display, &time::format_description::well_known::Rfc3339)
                .expect("display output should be valid RFC 3339");
    }
}
