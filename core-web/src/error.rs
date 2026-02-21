use aide::{
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse},
    operation::OperationOutput,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum AppError {
    Internal(anyhow::Error),
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    TooManyRequests(String),
    UnprocessableEntity(String),
    Validation {
        message: String,
        errors: HashMap<String, Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Vec<String>>>,
}

impl ErrorResponse {
    fn from_parts(
        status: StatusCode,
        message: Option<String>,
        error_code: Option<&str>,
        errors: Option<HashMap<String, Vec<String>>>,
    ) -> Self {
        let message = message
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| status_title(status).to_string());
        Self {
            message,
            error_code: error_code.map(str::to_string),
            errors,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, error_code, errors) = match self {
            AppError::Internal(e) => {
                tracing::error!("Internal server error: {:#}", e);
                let message = if cfg!(debug_assertions) {
                    Some(e.to_string())
                } else {
                    None
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message,
                    Some("INTERNAL_ERROR"),
                    None,
                )
            }
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, Some(m), Some("NOT_FOUND"), None),
            AppError::BadRequest(m) => {
                (StatusCode::BAD_REQUEST, Some(m), Some("BAD_REQUEST"), None)
            }
            AppError::Unauthorized(m) => (
                StatusCode::UNAUTHORIZED,
                Some(m),
                Some("UNAUTHORIZED"),
                None,
            ),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, Some(m), Some("FORBIDDEN"), None),
            AppError::TooManyRequests(m) => (
                StatusCode::TOO_MANY_REQUESTS,
                Some(m),
                Some("RATE_LIMITED"),
                None,
            ),
            AppError::UnprocessableEntity(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Some(m),
                Some("VALIDATION_ERROR"),
                None,
            ),
            AppError::Validation { message, errors } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Some(message),
                Some("VALIDATION_ERROR"),
                Some(errors),
            ),
        };

        let payload = ErrorResponse::from_parts(status, message, error_code, errors);
        (status, Json(payload)).into_response()
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
            (
                Some(400),
                error_response(ctx, StatusCode::BAD_REQUEST, "BAD_REQUEST", false),
            ),
            (
                Some(401),
                error_response(ctx, StatusCode::UNAUTHORIZED, "UNAUTHORIZED", false),
            ),
            (
                Some(403),
                error_response(ctx, StatusCode::FORBIDDEN, "FORBIDDEN", false),
            ),
            (
                Some(404),
                error_response(ctx, StatusCode::NOT_FOUND, "NOT_FOUND", false),
            ),
            (
                Some(422),
                error_response(
                    ctx,
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    true,
                ),
            ),
            (
                Some(429),
                error_response(ctx, StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", false),
            ),
            (
                Some(500),
                error_response(
                    ctx,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    false,
                ),
            ),
        ]
    }
}

fn status_title(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "Bad Request",
        StatusCode::UNAUTHORIZED => "Unauthorized",
        StatusCode::FORBIDDEN => "Forbidden",
        StatusCode::NOT_FOUND => "Not Found",
        StatusCode::UNPROCESSABLE_ENTITY => "Unprocessable Entity",
        StatusCode::TOO_MANY_REQUESTS => "Too Many Requests",
        StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
        _ => "Error",
    }
}

fn error_response(
    ctx: &mut GenContext,
    status: StatusCode,
    error_code: &str,
    include_validation_errors: bool,
) -> OpenApiResponse {
    let schema = ctx.schema.subschema_for::<ErrorResponse>();
    let mut example = serde_json::json!({
        "message": status_title(status),
        "error_code": error_code
    });
    if include_validation_errors {
        example["message"] = serde_json::Value::String("Validation failed".to_string());
        example["errors"] = serde_json::json!({
            "field": ["Validation failed"]
        });
    }
    OpenApiResponse {
        description: status_title(status).to_string(),
        content: [(
            "application/json".to_string(),
            aide::openapi::MediaType {
                schema: Some(aide::openapi::SchemaObject {
                    json_schema: schema,
                    example: Some(example),
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
