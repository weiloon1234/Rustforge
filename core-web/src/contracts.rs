use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

use crate::extract::{AsyncValidatedJson, ValidatedJson};

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
