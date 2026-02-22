use core_web::contracts::{rustforge_contract, rustforge_string_rule_type};
use core_web::extract::AsyncValidate;
use schemars::schema_for;
use validator::Validate;

#[rustforge_contract]
struct DemoInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(alpha_dash)]
    username: String,

    #[rf(alpha_dash)]
    optional_handle: Option<String>,

    #[rf(phonenumber(field = "contact_country_iso2"))]
    #[rf(openapi(hint = "Store raw input; server validates by country."))]
    phone: String,

    contact_country_iso2: String,
}

#[test]
fn rustforge_contract_runtime_validation_works() {
    let ok = DemoInput {
        username: "user_name-1".to_string(),
        optional_handle: None,
        phone: "+60123456789".to_string(),
        contact_country_iso2: "MY".to_string(),
    };
    assert!(ok.validate().is_ok());

    let bad = DemoInput {
        username: "bad name!".to_string(),
        optional_handle: Some("ok_value".to_string()),
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

#[rustforge_contract]
struct NestedChildInput {
    #[rf(length(min = 1, max = 20))]
    label: String,
}

#[rustforge_contract]
struct NestedParentInput {
    #[rf(nested)]
    child: NestedChildInput,
}

#[rustforge_contract]
struct PasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    password: String,
    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password"))]
    password_confirmation: String,
}

#[test]
fn rustforge_contract_nested_and_must_match_work() {
    let nested_bad = NestedParentInput {
        child: NestedChildInput {
            label: "".to_string(),
        },
    };
    assert!(nested_bad.validate().is_err());

    let pw_bad = PasswordUpdateInput {
        password: "Password123".to_string(),
        password_confirmation: "Password124".to_string(),
    };
    let err = pw_bad.validate().expect_err("must_match should fail");
    let text = format!("{err:?}");
    assert!(text.contains("password_confirmation"));
}

fn validate_username_wrapper(value: &str) -> Result<(), validator::ValidationError> {
    core_web::rules::required_trimmed(value)?;
    core_web::rules::alpha_dash(value)?;
    if value != value.to_ascii_lowercase() {
        let mut err = validator::ValidationError::new("lowercase");
        err.message = Some("Username must be lowercase.".into());
        return Err(err);
    }
    Ok(())
}

rustforge_string_rule_type! {
    /// Username wrapper type (project-level SSOT example).
    pub struct UsernameString {
        #[validate(custom(function = "validate_username_wrapper"))]
        #[rf(length(min = 3, max = 64))]
        #[rf(alpha_dash)]
        #[rf(openapi(description = "Lowercase username using letters, numbers, _ and -.", example = "admin_user"))]
    }
}

#[rustforge_contract]
struct WrapperDemoInput {
    #[rf(nested)]
    username: UsernameString,
}

#[rustforge_contract]
struct ContainsDemoInput {
    #[rf(contains(pattern = "@"))]
    email_like: String,
    #[rf(does_not_contain(pattern = " "))]
    username: String,
}

#[rustforge_contract]
struct ExampleValueInput {
    #[rf(openapi(example = 42))]
    count: i64,
    #[rf(openapi(example = false))]
    enabled: bool,
}

#[rustforge_contract]
struct OverrideMessageInput {
    #[rf(length(min = 3, max = 32), message = "Field-level default message")]
    #[rf(alpha_dash(message = "Alpha-dash failed"))]
    username: String,
}

fn validate_pair_not_same(value: &SchemaRuleInput) -> Result<(), validator::ValidationError> {
    if value.left == value.right {
        let mut err = validator::ValidationError::new("schema");
        err.message = Some("left and right must be different".into());
        return Err(err);
    }
    Ok(())
}

#[rustforge_contract]
#[rf(schema(function = "validate_pair_not_same", skip_on_field_errors = false))]
struct SchemaRuleInput {
    #[rf(length(min = 1, max = 10))]
    left: String,
    #[rf(length(min = 1, max = 10))]
    right: String,
}

#[rustforge_contract]
struct AsyncRuleInput {
    #[rf(async_unique(table = "admin", column = "username"))]
    username: String,
}

#[rustforge_contract]
struct AsyncRuleAdvancedInput {
    id: i64,
    tenant_id: i64,
    username: String,
    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "id"),
        where_eq(column = "tenant_id", field = "tenant_id"),
        where_null(column = "deleted_at")
    ))]
    scoped_username: String,
}

#[test]
fn rustforge_string_rule_type_wrapper_works() {
    let ok = WrapperDemoInput {
        username: UsernameString::from("admin_user".to_string()),
    };
    assert!(ok.validate().is_ok());
    assert_eq!(ok.username.as_str(), "admin_user");

    let bad = WrapperDemoInput {
        username: UsernameString::from("Bad Name".to_string()),
    };
    let err = bad.validate().expect_err("expected username validation failure");
    let text = format!("{err:?}");
    assert!(text.contains("username"));

    let schema_json = serde_json::to_value(schema_for!(UsernameString)).expect("schema json");
    let text = schema_json.to_string();
    assert!(text.contains("\"minLength\":3"));
    assert!(text.contains("\"maxLength\":64"));
    assert!(text.contains("^[A-Za-z0-9_-]+$"));
    assert!(text.contains("Lowercase username"));
    assert!(text.contains("\"x-rf-rules\""));
}

#[test]
fn rustforge_contract_contains_and_does_not_contain_work() {
    let ok = ContainsDemoInput {
        email_like: "hello@example.com".to_string(),
        username: "admin_user".to_string(),
    };
    assert!(ok.validate().is_ok());

    let bad = ContainsDemoInput {
        email_like: "hello.example.com".to_string(),
        username: "admin user".to_string(),
    };
    let err = bad.validate().expect_err("contains/does_not_contain should fail");
    let text = format!("{err:?}");
    assert!(text.contains("email_like"));
    assert!(text.contains("username"));

    let root = schema_for!(ContainsDemoInput);
    let props = &root
        .schema
        .object
        .as_ref()
        .expect("object schema")
        .properties;

    let email_like = match props.get("email_like").expect("email_like property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("email_like should be object schema"),
    };
    let email_like_desc = email_like
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(email_like_desc.contains("Must contain `@`"));
    assert!(email_like.extensions.contains_key("x-rf-rules"));

    let username = match props.get("username").expect("username property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("username should be object schema"),
    };
    let username_desc = username
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(username_desc.contains("Must not contain ` `"));
    assert!(username.extensions.contains_key("x-rf-rules"));
}

#[test]
fn rustforge_contract_non_string_openapi_examples_work() {
    let root = schema_for!(ExampleValueInput);
    let props = &root
        .schema
        .object
        .as_ref()
        .expect("object schema")
        .properties;

    let count = match props.get("count").expect("count property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("count should be object schema"),
    };
    let count_examples = &count
        .metadata
        .as_ref()
        .expect("count metadata")
        .examples;
    assert_eq!(count_examples.first(), Some(&serde_json::json!(42)));

    let enabled = match props.get("enabled").expect("enabled property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("enabled should be object schema"),
    };
    let enabled_examples = &enabled
        .metadata
        .as_ref()
        .expect("enabled metadata")
        .examples;
    assert_eq!(enabled_examples.first(), Some(&serde_json::json!(false)));
}

#[test]
fn rustforge_contract_per_rule_override_message_works() {
    let bad = OverrideMessageInput {
        username: "!".to_string(),
    };
    let err = bad.validate().expect_err("expected both length and alpha_dash errors");
    let field_errors = err.field_errors();
    let username_errors = field_errors
        .get("username")
        .expect("username field errors present");
    let messages = username_errors
        .iter()
        .filter_map(|item| item.message.as_ref().map(ToString::to_string))
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|m| m == "Alpha-dash failed"));
    assert!(messages.iter().any(|m| m == "Field-level default message"));
}

#[test]
fn rustforge_contract_schema_rule_via_rf_works() {
    let bad = SchemaRuleInput {
        left: "same".to_string(),
        right: "same".to_string(),
    };
    let err = bad.validate().expect_err("schema rule should fail");
    assert!(err.errors().contains_key("__all__"));
}

#[test]
fn rustforge_contract_async_rule_impl_is_generated() {
    fn assert_async_validate<T: AsyncValidate>() {}
    assert_async_validate::<AsyncRuleInput>();

    let root = schema_for!(AsyncRuleInput);
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
    let desc = username
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(desc.contains("Must be unique in `admin.username`"));
    assert!(username.extensions.contains_key("x-rf-rules"));
}

#[test]
fn rustforge_contract_async_rule_modifiers_compile_and_schema_exists() {
    fn assert_async_validate<T: AsyncValidate>() {}
    assert_async_validate::<AsyncRuleAdvancedInput>();

    let root = schema_for!(AsyncRuleAdvancedInput);
    let props = &root
        .schema
        .object
        .as_ref()
        .expect("object schema")
        .properties;
    let field = match props.get("scoped_username").expect("scoped_username property") {
        schemars::schema::Schema::Object(obj) => obj,
        _ => panic!("scoped_username should be object schema"),
    };
    let desc = field
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
        .unwrap_or_default();
    assert!(desc.contains("Must be unique in `admin.username`"));
    assert!(field.extensions.contains_key("x-rf-rules"));
}
