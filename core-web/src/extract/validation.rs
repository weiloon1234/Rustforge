use std::collections::HashMap;

use anyhow::Result;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

pub type ValidationErrorMap = HashMap<String, Vec<String>>;

#[async_trait::async_trait]
pub trait AsyncValidate {
    async fn validate_async(&self, db: &sqlx::PgPool) -> Result<(), ValidationErrors>;
}

pub fn transform_validation_errors(errors: ValidationErrors) -> ValidationErrorMap {
    let mut out = HashMap::new();
    flatten_validation_errors(None, &errors, &mut out);
    out
}

fn flatten_validation_errors(
    prefix: Option<&str>,
    errors: &ValidationErrors,
    out: &mut ValidationErrorMap,
) {
    for (field, kind) in errors.errors() {
        let field = field.as_ref();
        let path = match prefix {
            Some(parent) if !parent.is_empty() => format!("{parent}.{field}"),
            _ => field.to_string(),
        };

        match kind {
            ValidationErrorsKind::Field(items) => {
                let messages = items.iter().map(validation_message).collect::<Vec<_>>();
                out.entry(path).or_default().extend(messages);
            }
            ValidationErrorsKind::Struct(nested) => {
                flatten_validation_errors(Some(&path), nested, out);
            }
            ValidationErrorsKind::List(nested_list) => {
                for (index, nested) in nested_list {
                    let nested_path = format!("{path}.{index}");
                    flatten_validation_errors(Some(&nested_path), nested, out);
                }
            }
        }
    }
}

fn validation_message(error: &ValidationError) -> String {
    error
        .message
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| core_i18n::t("Invalid"))
}

pub fn apply_json_request_body_schema<T: schemars::JsonSchema>(
    ctx: &mut aide::generate::GenContext,
    operation: &mut aide::openapi::Operation,
) {
    let schema = ctx.schema.subschema_for::<T>();
    let schema_obj = schema.into_object();

    let json_val = serde_json::to_value(&schema_obj).expect("Failed to serialize schema");
    let aide_schema: aide::openapi::SchemaObject =
        serde_json::from_value(json_val).expect("Failed to convert schema to aide type");

    let body = operation.request_body.get_or_insert_with(|| {
        aide::openapi::ReferenceOr::Item(aide::openapi::RequestBody::default())
    });

    if let aide::openapi::ReferenceOr::Item(body) = body {
        body.content.insert(
            "application/json".to_string(),
            aide::openapi::MediaType {
                schema: Some(aide_schema),
                ..Default::default()
            },
        );
    }
}
