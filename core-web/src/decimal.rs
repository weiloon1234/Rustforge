use std::fmt;
use std::ops::{Add, Deref, Div, Mul, Sub};
use std::str::FromStr;

use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Precision-safe decimal newtype that serializes as a string on the wire.
///
/// Wraps `rust_decimal::Decimal` with built-in `JsonSchema` and `TS` impls
/// so contract structs need zero per-field annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub rust_decimal::Decimal);

impl Decimal {
    pub const ZERO: Self = Self(rust_decimal::Decimal::ZERO);
    pub const ONE: Self = Self(rust_decimal::Decimal::ONE);
    pub const TWO: Self = Self(rust_decimal::Decimal::TWO);
    pub const TEN: Self = Self(rust_decimal::Decimal::TEN);
    pub const ONE_HUNDRED: Self = Self(rust_decimal::Decimal::ONE_HUNDRED);
    pub const ONE_THOUSAND: Self = Self(rust_decimal::Decimal::ONE_THOUSAND);
    pub const NEGATIVE_ONE: Self = Self(rust_decimal::Decimal::NEGATIVE_ONE);
}

impl Deref for Decimal {
    type Target = rust_decimal::Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<rust_decimal::Decimal> for Decimal {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self(value)
    }
}

impl From<Decimal> for rust_decimal::Decimal {
    fn from(value: Decimal) -> Self {
        value.0
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct DecimalVisitor;

impl<'de> Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a decimal encoded as string or number")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        rust_decimal::Decimal::from_str(value)
            .map(Decimal)
            .map_err(|err| E::custom(format!("invalid decimal: {err}")))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal(rust_decimal::Decimal::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal(rust_decimal::Decimal::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        rust_decimal::Decimal::try_from(value)
            .map(Decimal)
            .map_err(|err| E::custom(format!("invalid decimal from float: {err}")))
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DecimalVisitor)
    }
}

impl JsonSchema for Decimal {
    fn schema_name() -> String {
        "Decimal".to_string()
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

impl ts_rs::TS for Decimal {
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

// Arithmetic ops delegating to inner type.

impl Add for Decimal {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Decimal {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Decimal {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for Decimal {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    use super::Decimal;

    #[test]
    fn serde_round_trip_string() {
        let d = Decimal(rust_decimal::Decimal::new(12345, 2)); // 123.45
        let json = serde_json::to_string(&d).expect("serialize");
        assert_eq!(json, "\"123.45\"");
        let back: Decimal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(d, back);
    }

    #[test]
    fn deserializes_from_number() {
        let from_number: Decimal = serde_json::from_str("123.45").expect("deserialize number");
        let from_string: Decimal =
            serde_json::from_str("\"123.45\"").expect("deserialize string");
        assert_eq!(from_number, from_string);
    }

    #[test]
    fn schema_is_string() {
        let schema = schema_for!(Decimal);
        let text = serde_json::to_string(&schema).expect("schema json");
        assert!(text.contains("\"type\":\"string\""));
    }

    #[test]
    fn ts_output_is_string() {
        assert_eq!(<Decimal as ts_rs::TS>::name(), "string");
    }

    #[test]
    fn arithmetic_ops() {
        let a = Decimal(rust_decimal::Decimal::new(100, 0));
        let b = Decimal(rust_decimal::Decimal::new(30, 0));
        assert_eq!((a + b).0, rust_decimal::Decimal::new(130, 0));
        assert_eq!((a - b).0, rust_decimal::Decimal::new(70, 0));
        assert_eq!((a * b).0, rust_decimal::Decimal::new(3000, 0));
        assert_eq!((a / b).0, rust_decimal::Decimal::new(100, 0) / rust_decimal::Decimal::new(30, 0));
    }
}
