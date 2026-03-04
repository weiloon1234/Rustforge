use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Precision-safe API identifier for Snowflake-style i64 IDs.
///
/// Wire format is always JSON string to prevent precision loss in JavaScript clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SnowflakeId(i64);

impl SnowflakeId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for SnowflakeId {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<SnowflakeId> for i64 {
    fn from(value: SnowflakeId) -> Self {
        value.0
    }
}

impl From<SnowflakeId> for String {
    fn from(value: SnowflakeId) -> Self {
        value.0.to_string()
    }
}

impl fmt::Display for SnowflakeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for SnowflakeId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.trim().parse::<i64>()?))
    }
}

impl Serialize for SnowflakeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct SnowflakeIdVisitor;

impl<'de> Visitor<'de> for SnowflakeIdVisitor {
    type Value = SnowflakeId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an i64 snowflake id encoded as string or integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SnowflakeId::new(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = i64::try_from(value).map_err(|_| E::custom("snowflake id is out of i64 range"))?;
        Ok(SnowflakeId::new(id))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse::<i64>()
            .map(SnowflakeId::new)
            .map_err(|err| E::custom(format!("invalid snowflake id: {err}")))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

impl<'de> Deserialize<'de> for SnowflakeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SnowflakeIdVisitor)
    }
}

impl JsonSchema for SnowflakeId {
    fn schema_name() -> String {
        "SnowflakeId".to_string()
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

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    use super::SnowflakeId;

    #[test]
    fn serializes_as_json_string() {
        let id = SnowflakeId::new(287_598_687_431_950_336);
        let encoded = serde_json::to_string(&id).expect("serialize snowflake id");
        assert_eq!(encoded, "\"287598687431950336\"");
    }

    #[test]
    fn deserializes_from_string_and_number() {
        let from_string: SnowflakeId =
            serde_json::from_str("\"287598687431950336\"").expect("decode id string");
        let from_number: SnowflakeId =
            serde_json::from_str("287598687431950336").expect("decode id number");

        assert_eq!(from_string, from_number);
        assert_eq!(from_string.as_i64(), 287_598_687_431_950_336);
    }

    #[test]
    fn round_trip_with_i64() {
        let raw = 123_456_789_i64;
        let id = SnowflakeId::from(raw);
        let back: i64 = id.into();
        assert_eq!(back, raw);
    }

    #[test]
    fn schema_is_string() {
        let schema = schema_for!(SnowflakeId);
        let text = serde_json::to_string(&schema).expect("schema json");
        assert!(text.contains("\"type\":\"string\""));
    }
}
