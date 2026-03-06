use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::{Dummy, TypeVisitor, TS};
use validator::{
    Validate, ValidateContains, ValidateEmail, ValidateLength, ValidateRequired, ValidateUrl,
    ValidationErrors,
};

/// Tri-state PATCH field:
/// - `Missing`: field omitted from the payload
/// - `Null`: field explicitly set to null
/// - `Value(T)`: field provided with a concrete value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<T> Patch<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    pub fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }

    pub fn into_value(self) -> Option<T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }

    pub fn map_value<U>(self, f: impl FnOnce(T) -> U) -> Patch<U> {
        match self {
            Self::Missing => Patch::Missing,
            Self::Null => Patch::Null,
            Self::Value(value) => Patch::Value(f(value)),
        }
    }
}

impl<T> From<T> for Patch<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T> Serialize for Patch<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Missing | Self::Null => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T> Validate for Patch<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Self::Value(value) => value.validate(),
            Self::Missing | Self::Null => Ok(()),
        }
    }
}

impl<T> ValidateEmail for Patch<T>
where
    T: ValidateEmail,
{
    fn as_email_string(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.as_value().and_then(T::as_email_string)
    }
}

impl<T> ValidateLength<u64> for Patch<T>
where
    T: ValidateLength<u64>,
{
    fn length(&self) -> Option<u64> {
        self.as_value().and_then(T::length)
    }
}

impl<T> ValidateUrl for Patch<T>
where
    T: ValidateUrl,
{
    fn as_url_string(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.as_value().and_then(T::as_url_string)
    }
}

impl<T> ValidateContains for Patch<T>
where
    T: ValidateContains,
{
    fn validate_contains(&self, needle: &str) -> bool {
        self.as_value()
            .map(|value| value.validate_contains(needle))
            .unwrap_or(true)
    }
}

impl<T> ValidateRequired for Patch<T> {
    fn is_some(&self) -> bool {
        matches!(self, Self::Value(_))
    }
}

impl<T> JsonSchema for Patch<T>
where
    T: JsonSchema,
{
    fn is_referenceable() -> bool {
        false
    }

    fn schema_name() -> String {
        format!("Patch_{}", T::schema_name())
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!("Patch<{}>", T::schema_id()))
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<T>>::json_schema(generator)
    }

    fn _schemars_private_non_optional_json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<T>>::_schemars_private_non_optional_json_schema(generator)
    }
}

impl<T> TS for Patch<T>
where
    T: TS,
{
    type WithoutGenerics = Patch<Dummy>;

    fn name() -> String {
        format!("{} | null", T::name())
    }

    fn inline() -> String {
        format!("{} | null", T::inline())
    }

    fn visit_dependencies(v: &mut impl TypeVisitor)
    where
        Self: 'static,
    {
        T::visit_dependencies(v);
    }

    fn visit_generics(v: &mut impl TypeVisitor)
    where
        Self: 'static,
    {
        T::visit_generics(v);
        v.visit::<T>();
    }

    fn decl() -> String {
        panic!("{} cannot be declared", Self::name())
    }

    fn decl_concrete() -> String {
        panic!("{} cannot be declared", Self::name())
    }

    fn inline_flattened() -> String {
        panic!("{} cannot be flattened", Self::name())
    }
}
