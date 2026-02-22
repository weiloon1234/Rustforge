use core_web::contracts::rustforge_contract;
use schemars::schema_for;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Clone, Deserialize, Validate, schemars::JsonSchema)]
struct DemoInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(rule = "alpha_dash")]
    username: String,

    #[rf(rule = "phonenumber", field = "contact_country_iso2")]
    #[rf(openapi_hint = "Store raw input; server validates by country.")]
    phone: String,

    contact_country_iso2: String,
}

#[test]
fn rustforge_contract_runtime_validation_works() {
    let ok = DemoInput {
        username: "user_name-1".to_string(),
        phone: "+60123456789".to_string(),
        contact_country_iso2: "MY".to_string(),
    };
    assert!(ok.validate().is_ok());

    let bad = DemoInput {
        username: "bad name!".to_string(),
        phone: "not-a-phone".to_string(),
        contact_country_iso2: "MY".to_string(),
    };
    let err = bad.validate().expect_err("expected validation failure");
    let text = format!("{err:?}");
    assert!(text.contains("username"));
    assert!(text.contains("phone"));
}

#[test]
fn rustforge_contract_schema_contains_constraints_and_extensions() {
    let root = schema_for!(DemoInput);
    let props = &root
        .schema
        .object
        .as_ref()
        .expect("object schema")
        .properties;

    let username = match props.get("username").expect("username property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("username should be object schema"),
    };

    let string_validation = username
        .string
        .as_ref()
        .expect("username string validation");
    assert_eq!(string_validation.min_length, Some(3));
    assert_eq!(string_validation.max_length, Some(32));
    assert_eq!(
        string_validation.pattern.as_deref(),
        Some("^[A-Za-z0-9_-]+$")
    );
    assert!(username.extensions.contains_key("x-rf-rules"));
    assert!(username.extensions.contains_key("x-rf-rule-summary"));
    let username_desc = username
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(username_desc.contains("Letters, numbers, underscore"));

    let phone = match props.get("phone").expect("phone property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("phone should be object schema"),
    };
    assert_eq!(phone.format.as_deref(), Some("phone"));
    let phone_desc = phone
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(phone_desc.contains("contact_country_iso2"));
    assert!(phone_desc.contains("Store raw input"));
}
