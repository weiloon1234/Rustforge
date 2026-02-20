use aide::{
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse},
    operation::OperationOutput,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Internal(anyhow::Error),
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    TooManyRequests(String),
    UnprocessableEntity(String),
}

use crate::response::ApiResponse;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg, error_code) = match self {
            AppError::Internal(e) => {
                tracing::error!("Internal server error: {:#}", e);
                // In Development (debug assertions), return the actual error.
                if cfg!(debug_assertions) {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        e.to_string(),
                        "INTERNAL_ERROR",
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                        "INTERNAL_ERROR",
                    )
                }
            }
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m, "NOT_FOUND"),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m, "BAD_REQUEST"),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m, "UNAUTHORIZED"),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m, "FORBIDDEN"),
            AppError::TooManyRequests(m) => (StatusCode::TOO_MANY_REQUESTS, m, "RATE_LIMITED"),
            AppError::UnprocessableEntity(m) => {
                (StatusCode::UNPROCESSABLE_ENTITY, m, "VALIDATION_ERROR")
            }
        };

        // Standardized Errors are just success: false, data: None
        ApiResponse::error(
            status,
            &msg,
            Some(error_code.to_string()),
            None, // Detail errors map could be passed if AppError supported it
        )
        .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}

impl OperationOutput for AppError {
    type Inner = Self;

    fn operation_response(
        _ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        None
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        vec![
            (Some(400), error_response(ctx, "Bad request")),
            (Some(401), error_response(ctx, "Unauthorized")),
            (Some(403), error_response(ctx, "Forbidden")),
            (Some(404), error_response(ctx, "Not found")),
            (Some(422), error_response(ctx, "Validation failed")),
            (Some(429), error_response(ctx, "Rate limited")),
            (Some(500), error_response(ctx, "Internal server error")),
        ]
    }
}

fn error_response(ctx: &mut GenContext, description: &str) -> OpenApiResponse {
    let schema = ctx.schema.subschema_for::<ApiResponse<()>>();
    OpenApiResponse {
        description: description.to_string(),
        content: [(
            "application/json".to_string(),
            aide::openapi::MediaType {
                schema: Some(aide::openapi::SchemaObject {
                    json_schema: schema,
                    example: None,
                    external_docs: None,
                }),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    }
}
