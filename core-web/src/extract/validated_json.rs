use axum::{
    extract::{FromRequest, Request},
    Json,
};
use core_i18n::t;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;
use crate::extract::validation::{apply_json_request_body_schema, transform_validation_errors};

pub struct ValidatedJson<T>(pub T);

// #[axum::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Parse JSON
        let Json(data) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("{}: {}", t("Invalid JSON"), e)))?;

        if let Err(e) = data.validate() {
            let errors = transform_validation_errors(e);
            return Err(AppError::Validation {
                message: t("Validation failed"),
                errors,
            });
        }

        Ok(ValidatedJson(data))
    }
}

// Trait to get DB from State (must be implemented by AppState)
pub trait GetDb {
    fn db(&self) -> &sqlx::PgPool;
}

// Implement OperationInput for Aide
impl<T> aide::OperationInput for ValidatedJson<T>
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
