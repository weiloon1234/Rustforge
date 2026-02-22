#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinRuleArgs {
    /// No arguments: `#[rf(strong_password)]`
    /// With message only: `#[rf(strong_password(message = "..."))]`
    None,
    /// Accepts list of string values: `#[rf(one_of("a", "b"))]`
    Values,
    /// Accepts `format = "..."`: `#[rf(date(format = "..."))]`
    Format,
    /// Accepts `field = "..."`: `#[rf(phonenumber(field = "country_iso2"))]`
    Field,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinRuleKind {
    CustomFnPath(&'static str),
    GeneratedOneOf,
    GeneratedNoneOf,
    GeneratedDate,
    GeneratedDateTime,
    PhoneNumberByIso2Field,
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinRuleMeta {
    pub key: &'static str,
    pub kind: BuiltinRuleKind,
    pub args: BuiltinRuleArgs,
    pub default_code: &'static str,
    pub default_message: &'static str,
    pub openapi_description_template: &'static str,
    pub pattern: Option<&'static str>,
    pub format: Option<&'static str>,
}

const BUILTIN_RULES: &[BuiltinRuleMeta] = &[
    BuiltinRuleMeta {
        key: "required_trimmed",
        kind: BuiltinRuleKind::CustomFnPath("core_web::rules::required_trimmed"),
        args: BuiltinRuleArgs::None,
        default_code: "required_trimmed",
        default_message: "This field is required.",
        openapi_description_template: "Must not be empty or whitespace-only.",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "alpha_dash",
        kind: BuiltinRuleKind::CustomFnPath("core_web::rules::alpha_dash"),
        args: BuiltinRuleArgs::None,
        default_code: "alpha_dash",
        default_message: "Only letters, numbers, underscores, and dashes are allowed.",
        openapi_description_template: "Letters, numbers, underscore (_), and hyphen (-) only.",
        pattern: Some("^[A-Za-z0-9_-]+$"),
        format: None,
    },
    BuiltinRuleMeta {
        key: "lowercase_slug",
        kind: BuiltinRuleKind::CustomFnPath("core_web::rules::lowercase_slug"),
        args: BuiltinRuleArgs::None,
        default_code: "lowercase_slug",
        default_message: "Must be a lowercase slug (letters/numbers separated by single dashes).",
        openapi_description_template:
            "Lowercase slug using letters and numbers separated by single hyphens.",
        pattern: Some("^[a-z0-9]+(?:-[a-z0-9]+)*$"),
        format: None,
    },
    BuiltinRuleMeta {
        key: "strong_password",
        kind: BuiltinRuleKind::CustomFnPath("core_web::rules::strong_password"),
        args: BuiltinRuleArgs::None,
        default_code: "strong_password",
        default_message: "Password is too weak.",
        openapi_description_template:
            "Strong password required (uppercase, lowercase, and number).",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "one_of",
        kind: BuiltinRuleKind::GeneratedOneOf,
        args: BuiltinRuleArgs::Values,
        default_code: "one_of",
        default_message: "Value is not in the allowed list.",
        openapi_description_template: "Allowed values: {values}.",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "none_of",
        kind: BuiltinRuleKind::GeneratedNoneOf,
        args: BuiltinRuleArgs::Values,
        default_code: "none_of",
        default_message: "Value is in the blocked list.",
        openapi_description_template: "Blocked values: {values}.",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "date",
        kind: BuiltinRuleKind::GeneratedDate,
        args: BuiltinRuleArgs::Format,
        default_code: "date",
        default_message: "Invalid date format.",
        openapi_description_template: "Date string in format `{format}`.",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "datetime",
        kind: BuiltinRuleKind::GeneratedDateTime,
        args: BuiltinRuleArgs::Format,
        default_code: "datetime",
        default_message: "Invalid datetime format.",
        openapi_description_template: "Datetime string in format `{format}`.",
        pattern: None,
        format: None,
    },
    BuiltinRuleMeta {
        key: "phonenumber",
        kind: BuiltinRuleKind::PhoneNumberByIso2Field,
        args: BuiltinRuleArgs::Field,
        default_code: "phonenumber",
        default_message: "Invalid phone number for selected country.",
        openapi_description_template:
            "Phone number validated against ISO2 country code from `{field}`.",
        pattern: None,
        format: Some("phone"),
    },
];

pub fn builtin_rule_meta(key: &str) -> Option<&'static BuiltinRuleMeta> {
    BUILTIN_RULES.iter().find(|m| m.key == key)
}

pub fn builtin_rule_metas() -> &'static [BuiltinRuleMeta] {
    BUILTIN_RULES
}

pub fn render_template(template: &str, params: &[(&str, String)]) -> String {
    let mut out = template.to_string();
    for (k, v) in params {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}
