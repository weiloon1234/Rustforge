use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

use crate::extract::{AsyncValidatedJson, ValidatedJson};
pub use rustforge_contract_macros::rustforge_contract;

/// Generate a transparent `String` newtype wrapper for reusable request-contract semantics.
///
/// This macro keeps wrapper types concise while preserving SSOT:
/// - runtime validation on the wrapper field (`#[validate(...)]`)
/// - OpenAPI schema hints via `#[rf(...)]` on the wrapper field (through `#[rustforge_contract]`)
///
/// Use `#[rf(nested)]` on DTO fields that consume the generated wrapper type.
#[macro_export]
macro_rules! rustforge_string_rule_type {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(#[$field_attr:meta])*
        }
    ) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Deserialize,
            ::serde::Serialize
        )]
        #[serde(transparent)]
        $(#[$meta])*
        $vis struct $name(pub String);

        impl $name {
            pub fn new(value: String) -> Self {
                Self(value)
            }

            pub fn into_inner(self) -> String {
                self.0
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl ::std::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::convert::From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl ::std::convert::From<$name> for String {
            fn from(value: $name) -> Self {
                value.into_inner()
            }
        }

        const _: () = {
            #[::core_web::contracts::rustforge_contract]
            struct __RustforgeStringRuleTypeSchemaHelper {
                $(#[$field_attr])*
                value: String,
            }

            impl ::validator::Validate for $name {
                fn validate(&self) -> Result<(), ::validator::ValidationErrors> {
                    __RustforgeStringRuleTypeSchemaHelper {
                        value: self.0.clone(),
                    }
                    .validate()
                    .map_err(|errors| {
                        // The helper struct reports errors under a "value" key, which would
                        // surface as "field.value" in API responses. Promote them to an
                        // empty key so the flattener attaches them directly to the parent
                        // field name (e.g. "username" instead of "username.value").
                        let mut promoted = ::validator::ValidationErrors::new();
                        for (_key, kind) in errors.into_errors() {
                            if let ::validator::ValidationErrorsKind::Field(items) = kind {
                                for item in items {
                                    promoted.add("", item);
                                }
                            }
                        }
                        promoted
                    })
                }
            }

            impl ::schemars::JsonSchema for $name {
                fn schema_name() -> String {
                    stringify!($name).to_string()
                }

                fn json_schema(
                    generator: &mut ::schemars::gen::SchemaGenerator,
                ) -> ::schemars::schema::Schema {
                    let helper_schema =
                        <__RustforgeStringRuleTypeSchemaHelper as ::schemars::JsonSchema>::json_schema(
                            generator,
                        );
                    if let ::schemars::schema::Schema::Object(root_obj) = &helper_schema {
                        if let Some(obj_validation) = &root_obj.object {
                            if let Some(prop_schema) = obj_validation.properties.get("value") {
                                return prop_schema.clone();
                            }
                        }
                    }
                    helper_schema
                }

                fn _schemars_private_non_optional_json_schema(
                    generator: &mut ::schemars::gen::SchemaGenerator,
                ) -> ::schemars::schema::Schema {
                    <Self as ::schemars::JsonSchema>::json_schema(generator)
                }

                fn _schemars_private_is_option() -> bool {
                    false
                }
            }
        };
    };
}
pub use crate::rustforge_string_rule_type;

/// Canonical request DTO contract for API boundaries.
/// Use this as the app-level standard: one type drives runtime validation and OpenAPI schema.
pub trait RequestContract:
    DeserializeOwned + Validate + JsonSchema + Send + Sync + 'static
{
}

impl<T> RequestContract for T where
    T: DeserializeOwned + Validate + JsonSchema + Send + Sync + 'static
{
}

/// Canonical response DTO contract for OpenAPI output.
pub trait ResponseContract: Serialize + JsonSchema {}

impl<T> ResponseContract for T where T: Serialize + JsonSchema {}

/// Developer-friendly aliases for request extractors under the contract standard.
pub type ContractJson<T> = ValidatedJson<T>;
pub type AsyncContractJson<T> = AsyncValidatedJson<T>;
