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
        let path = match (prefix, field) {
            // Nested field under a parent, e.g. "address" + "city" → "address.city"
            (Some(parent), f) if !parent.is_empty() && !f.is_empty() => {
                format!("{parent}.{f}")
            }
            // Empty field key (from rustforge_string_rule_type) — use parent as-is
            (Some(parent), _) if !parent.is_empty() => parent.to_string(),
            // Top-level field
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
    // Prefer explicit message if set by the contract macro or custom validator.
    if let Some(msg) = &error.message {
        return msg.to_string();
    }

    // Auto-generate a human-readable message from the validator code and params.
    let p = &error.params;
    match error.code.as_ref() {
        "length" => {
            let min = p.get("min").and_then(|v| v.as_u64());
            let max = p.get("max").and_then(|v| v.as_u64());
            let equal = p.get("equal").and_then(|v| v.as_u64());
            if let Some(eq) = equal {
                return core_i18n::t(&format!("Must be exactly {eq} characters."));
            }
            match (min, max) {
                (Some(lo), Some(hi)) => {
                    core_i18n::t(&format!("Must be between {lo} and {hi} characters."))
                }
                (Some(lo), None) => {
                    core_i18n::t(&format!("Must be at least {lo} characters."))
                }
                (None, Some(hi)) => {
                    core_i18n::t(&format!("Must be at most {hi} characters."))
                }
                _ => core_i18n::t("Invalid length."),
            }
        }
        "range" => {
            let min = p.get("min").and_then(|v| v.as_f64());
            let max = p.get("max").and_then(|v| v.as_f64());
            match (min, max) {
                (Some(lo), Some(hi)) => {
                    core_i18n::t(&format!("Must be between {lo} and {hi}."))
                }
                (Some(lo), None) => core_i18n::t(&format!("Must be at least {lo}.")),
                (None, Some(hi)) => core_i18n::t(&format!("Must be at most {hi}.")),
                _ => core_i18n::t("Out of range."),
            }
        }
        "email" => core_i18n::t("Must be a valid email address."),
        "url" => core_i18n::t("Must be a valid URL."),
        "required" => core_i18n::t("This field is required."),
        "must_match" => core_i18n::t("Fields do not match."),
        "contains" => core_i18n::t("Missing required content."),
        "does_not_contain" => core_i18n::t("Contains disallowed content."),
        _ => core_i18n::t("Invalid"),
    }
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
