use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;

use schemars::JsonSchema;

#[derive(Serialize, JsonSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    // Provide a default for data if None, though skip_serializing_if might be better if optional.
    // User requested "data: ...extra data, can be array can be object".
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Vec<String>>>,

    // We don't serialize status_code in JSON usually, but we need it for IntoResponse
    #[serde(skip)]
    pub status_code: StatusCode,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Success Response (200 OK)
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            error_code: None,
            errors: None,
            status_code: StatusCode::OK,
        }
    }

    /// Created Response (201 Created)
    pub fn created(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            error_code: None,
            errors: None,
            status_code: StatusCode::CREATED,
        }
    }

    /// Generic "Make Response" (Laravel style)
    pub fn make_response(
        status: StatusCode,
        message: &str,
        data: Option<T>,
        error_code: Option<String>,
        errors: Option<HashMap<String, Vec<String>>>,
    ) -> Self {
        Self {
            success: status.is_success(),
            message: message.to_string(),
            data,
            error_code,
            errors,
            status_code: status,
        }
    }
}

// Error Response Helper (No Data)
impl ApiResponse<()> {
    pub fn error(
        status: StatusCode,
        message: &str,
        error_code: Option<String>,
        errors: Option<HashMap<String, Vec<String>>>,
    ) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
            error_code,
            errors,
            status_code: status,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = self.status_code;
        (status, Json(self)).into_response()
    }
}

use aide::{
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse},
    operation::OperationOutput,
};

impl<T> OperationOutput for ApiResponse<T>
where
    T: Serialize + JsonSchema,
{
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        let schema = ctx.schema.subschema_for::<Self>();
        Some(OpenApiResponse {
            description: "Successful response".to_string(),
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
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        if let Some(res) = Self::operation_response(ctx, operation) {
            vec![(Some(200), res)]
        } else {
            vec![]
        }
    }
}
