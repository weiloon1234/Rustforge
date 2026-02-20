use axum::{
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use core_db::common::active_record::ActiveRecord;

use crate::extract::validated_json::GetDb;
use crate::response::ApiResponse;

use serde::de::DeserializeOwned;

pub struct Model<T>(pub T);

// Removed #[async_trait] as per Axum 0.8
impl<T, S> FromRequestParts<S> for Model<T>
where
    T: ActiveRecord + Send + Sync,
    T::Id: DeserializeOwned + Send, // Ensure ID can be deserialized from Path
    S: Send + Sync + GetDb,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract ID from Path
        // We assume the path contains exactly one UUID parameter.
        // This leverages axum's Path extractor logic but simpler?
        // Actually, we can just delegate to Path<Uuid>

        let Path(id) = Path::<T::Id>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                // Return 404/400? If ID is malformed or missing.
                ApiResponse::error(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid ID: {}", e),
                    Some("INVALID_ID".to_string()),
                    None,
                )
                .into_response()
            })?;

        // 2. Fetch from DB
        let db = state.db();
        let record = T::find(db, id).await.map_err(|e| {
            tracing::error!("Model binding error: {}", e);
            ApiResponse::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                Some("INTERNAL_ERROR".to_string()),
                None,
            )
            .into_response()
        })?;

        match record {
            Some(r) => Ok(Model(r)),
            None => Err(ApiResponse::error(
                StatusCode::NOT_FOUND,
                "Resource not found",
                Some("NOT_FOUND".to_string()),
                None,
            )
            .into_response()),
        }
    }
}

impl<T> aide::OperationInput for Model<T>
where
    T: ActiveRecord + Send + Sync,
    T::Id: schemars::JsonSchema,
{
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        Path::<T::Id>::operation_input(ctx, operation);
    }
}
