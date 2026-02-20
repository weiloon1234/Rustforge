use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use core_i18n::t;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::extract::validation::{apply_json_request_body_schema, transform_validation_errors};
use crate::response::ApiResponse;

pub struct ValidatedJson<T>(pub T);

// #[axum::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Parse JSON
        let Json(data) = Json::<T>::from_request(req, state).await.map_err(|e| {
            ApiResponse::error(
                StatusCode::BAD_REQUEST,
                &format!("{}: {}", t("Invalid JSON"), e),
                Some("INVALID_JSON".to_string()),
                None,
            )
            .into_response()
        })?;

        if let Err(e) = data.validate() {
            let errors = transform_validation_errors(e);
            return Err(ApiResponse::error(
                StatusCode::UNPROCESSABLE_ENTITY,
                &t("Validation failed"),
                Some("VALIDATION_ERROR".to_string()),
                Some(errors),
            )
            .into_response());
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
