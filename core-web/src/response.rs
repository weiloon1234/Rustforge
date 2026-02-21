use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use schemars::JsonSchema;

#[derive(Serialize, JsonSchema)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip)]
    pub status_code: StatusCode,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Success Response (200 OK)
    pub fn success(data: T, message: &str) -> Self {
        let message = message.trim();
        Self {
            data,
            message: (!message.is_empty()).then(|| message.to_string()),
            status_code: StatusCode::OK,
        }
    }

    /// Created Response (201 Created)
    pub fn created(data: T, message: &str) -> Self {
        let message = message.trim();
        Self {
            data,
            message: (!message.is_empty()).then(|| message.to_string()),
            status_code: StatusCode::CREATED,
        }
    }

    /// Generic response constructor.
    pub fn make_response(status: StatusCode, data: T, message: Option<&str>) -> Self {
        let message = message
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        Self {
            data,
            message,
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
