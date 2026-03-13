use axum::extract::{FromRequest, Request};
use core_i18n::t;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;
use crate::extract::json_cleaner::clean_and_deserialize;
pub use crate::extract::validated_json::GetDb;
use crate::extract::validation::{
    apply_json_request_body_schema, transform_validation_errors, AsyncValidate,
};

pub struct AsyncValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for AsyncValidatedJson<T>
where
    T: DeserializeOwned + Validate + AsyncValidate + Send + Sync + 'static,
    S: Send + Sync + GetDb,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let data: T = clean_and_deserialize(req, state).await?;

        if let Err(e) = data.validate() {
            let errors = transform_validation_errors(e);
            return Err(AppError::Validation {
                message: t("Validation failed"),
                errors,
            });
        }

        if let Err(e) = data.validate_async(state.db()).await {
            let errors = transform_validation_errors(e);
            return Err(AppError::Validation {
                message: t("Validation failed"),
                errors,
            });
        }

        Ok(AsyncValidatedJson(data))
    }
}
impl<T> aide::OperationInput for AsyncValidatedJson<T>
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
