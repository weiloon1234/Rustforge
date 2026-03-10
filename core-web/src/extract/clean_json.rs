use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::error::AppError;
use crate::extract::json_cleaner::clean_and_deserialize;
use crate::extract::validation::apply_json_request_body_schema;

/// Drop-in replacement for `axum::Json<T>` that cleans string values before deserializing.
///
/// All string values are trimmed; empty/whitespace-only strings become `null`.
/// No validation is performed — use `ValidatedJson` or `AsyncValidatedJson` when validation is needed.
pub struct CleanJson<T>(pub T);

impl<T, S> FromRequest<S> for CleanJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(CleanJson(clean_and_deserialize(req, state).await?))
    }
}

impl<T> aide::OperationInput for CleanJson<T>
where
    T: schemars::JsonSchema,
{
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        apply_json_request_body_schema::<T>(ctx, operation);
    }
}
