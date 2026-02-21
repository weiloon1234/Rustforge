use aide::{
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse},
    operation::OperationOutput,
};
use axum::{
    http::{
        header::CONTENT_TYPE,
        HeaderValue, StatusCode,
    },
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
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Vec<String>>>,
}

impl ProblemDetails {
    fn from_parts(
        status: StatusCode,
        title: &str,
        detail: Option<String>,
        error_code: Option<&str>,
        errors: Option<HashMap<String, Vec<String>>>,
    ) -> Self {
        Self {
            kind: "about:blank".to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail: detail.filter(|value| !value.trim().is_empty()),
            error_code: error_code.map(str::to_string),
            errors,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail, error_code, errors) = match self {
            AppError::Internal(e) => {
                tracing::error!("Internal server error: {:#}", e);
                let detail = if cfg!(debug_assertions) {
                    Some(e.to_string())
                } else {
                    Some("Internal server error".to_string())
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    detail,
                    Some("INTERNAL_ERROR"),
                    None,
                )
            }
            AppError::NotFound(m) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                Some(m),
                Some("NOT_FOUND"),
                None,
            ),
            AppError::BadRequest(m) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                Some(m),
                Some("BAD_REQUEST"),
                None,
            ),
            AppError::Unauthorized(m) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                Some(m),
                Some("UNAUTHORIZED"),
                None,
            ),
            AppError::Forbidden(m) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                Some(m),
                Some("FORBIDDEN"),
                None,
            ),
            AppError::TooManyRequests(m) => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too Many Requests",
                Some(m),
                Some("RATE_LIMITED"),
                None,
            ),
            AppError::UnprocessableEntity(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Unprocessable Entity",
                Some(m),
                Some("VALIDATION_ERROR"),
                None,
            ),
            AppError::Validation { message, errors } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Unprocessable Entity",
                Some(message),
                Some("VALIDATION_ERROR"),
                Some(errors),
            ),
        };

        let problem = ProblemDetails::from_parts(status, title, detail, error_code, errors);
        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
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
            (Some(400), error_response(ctx, StatusCode::BAD_REQUEST, "BAD_REQUEST", false)),
            (
                Some(401),
                error_response(ctx, StatusCode::UNAUTHORIZED, "UNAUTHORIZED", false),
            ),
            (
                Some(403),
                error_response(ctx, StatusCode::FORBIDDEN, "FORBIDDEN", false),
            ),
            (Some(404), error_response(ctx, StatusCode::NOT_FOUND, "NOT_FOUND", false)),
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
    let schema = ctx.schema.subschema_for::<ProblemDetails>();
    let mut example = serde_json::json!({
        "type": "about:blank",
        "title": status_title(status),
        "status": status.as_u16(),
        "detail": status_title(status),
        "error_code": error_code
    });
    if include_validation_errors {
        example["errors"] = serde_json::json!({
            "field": ["Validation failed"]
        });
    }
    OpenApiResponse {
        description: status_title(status).to_string(),
        content: [(
            "application/problem+json".to_string(),
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
