use rustforge_contract_macros::rustforge_contract;
use serde::Deserialize;
use validator::Validate;

fn validate_pair(value: &DemoInput) -> Result<(), validator::ValidationError> {
    if value.a == value.b {
        let mut err = validator::ValidationError::new("schema");
        err.message = Some("a and b cannot match".into());
        return Err(err);
    }
    Ok(())
}

#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, schemars::JsonSchema)]
#[rf(schema(function = "validate_pair"))]
struct DemoInput {
    #[rf(async_unique(table = "demo", column = "a"))]
    #[rf(rule_override(rule = "async_unique", message = "Already used"))]
    a: String,
    b: String,
}

fn _assert_async_validate<T: core_web::extract::AsyncValidate>() {}

fn main() {
    _assert_async_validate::<DemoInput>();
}
