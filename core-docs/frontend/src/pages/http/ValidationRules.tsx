type RuleCategory = 'Sync' | 'Derive' | 'Async'

type RuleRow = {
    category: RuleCategory
    rule: string
    description: string
    sampleId: string
}

const RULE_ROWS: RuleRow[] = [
    {
        category: 'Sync',
        rule: 'required_trimmed',
        description: 'Rejects empty/whitespace-only strings.',
        sampleId: 'sample-required-trimmed',
    },
    {
        category: 'Sync',
        rule: 'alpha_dash',
        description: 'Allows only letters, numbers, underscores, and dashes.',
        sampleId: 'sample-alpha-dash',
    },
    {
        category: 'Sync',
        rule: 'lowercase_slug',
        description: 'Enforces lowercase slug format with single dashes.',
        sampleId: 'sample-lowercase-slug',
    },
    {
        category: 'Sync',
        rule: 'strong_password',
        description: 'Requires strong mixed password composition.',
        sampleId: 'sample-strong-password',
    },
    {
        category: 'Sync',
        rule: 'email',
        description: 'Validates email format using framework helper.',
        sampleId: 'sample-sync-email',
    },
    {
        category: 'Sync',
        rule: 'one_of',
        description: 'Allows only values from a fixed allow-list.',
        sampleId: 'sample-one-of-none-of',
    },
    {
        category: 'Sync',
        rule: 'none_of',
        description: 'Blocks values from a fixed deny-list.',
        sampleId: 'sample-one-of-none-of',
    },
    {
        category: 'Sync',
        rule: 'eq / gt / gte / lt / lte',
        description: 'Generic comparison rules (fixed value or other input).',
        sampleId: 'sample-comparison',
    },
    {
        category: 'Sync',
        rule: 'date / datetime',
        description: 'Validates date/datetime strings by custom format.',
        sampleId: 'sample-date-datetime',
    },
    {
        category: 'Sync',
        rule: 'date_* / datetime_*',
        description: 'Date and datetime compare helpers (eq/gt/gte/lt/lte).',
        sampleId: 'sample-date-comparison',
    },
    {
        category: 'Derive',
        rule: 'length',
        description: 'Built-in field length validation.',
        sampleId: 'sample-derive-length',
    },
    {
        category: 'Derive',
        rule: 'range',
        description: 'Built-in numeric range validation.',
        sampleId: 'sample-derive-range',
    },
    {
        category: 'Derive',
        rule: 'email',
        description: 'Built-in email validator in derive attribute.',
        sampleId: 'sample-derive-email',
    },
    {
        category: 'Derive',
        rule: 'must_match',
        description: 'Requires one field to match another field.',
        sampleId: 'sample-must-match',
    },
    {
        category: 'Derive',
        rule: 'phonenumber(field = ...)',
        description: 'Validates phone format against sibling country ISO2 field.',
        sampleId: 'sample-derive-phonenumber',
    },
    {
        category: 'Async',
        rule: 'Unique',
        description: 'Checks DB uniqueness constraints.',
        sampleId: 'sample-async-validate',
    },
    {
        category: 'Async',
        rule: 'Exists',
        description: 'Checks referenced DB value existence.',
        sampleId: 'sample-async-validate',
    },
    {
        category: 'Async',
        rule: 'NotExists',
        description: 'Checks referenced DB value is absent.',
        sampleId: 'sample-async-validate',
    },
    {
        category: 'Async',
        rule: 'PhoneByCountryIso2',
        description: 'Validates phone format by country using countries data.',
        sampleId: 'sample-async-validate',
    },
]

const SAMPLE_TITLE: Record<string, string> = {
    'sample-required-trimmed': 'required_trimmed',
    'sample-alpha-dash': 'alpha_dash',
    'sample-lowercase-slug': 'lowercase_slug',
    'sample-strong-password': 'strong_password',
    'sample-sync-email': 'sync email',
    'sample-one-of-none-of': 'one_of + none_of',
    'sample-comparison': 'eq/gt/gte/lt/lte',
    'sample-date-datetime': 'date + datetime',
    'sample-date-comparison': 'date_* + datetime_*',
    'sample-derive-length': 'derive length',
    'sample-derive-range': 'derive range',
    'sample-derive-email': 'derive email',
    'sample-must-match': 'must_match',
    'sample-derive-phonenumber': 'derive phonenumber',
    'sample-async-validate': 'AsyncValidate (Unique/Exists/NotExists/PhoneByCountryIso2)',
}

function jumpToSample(sampleId: string) {
    const element = document.getElementById(sampleId)
    if (!element) return
    element.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

export function ValidationRules() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Validation Rules</h1>
                <p className="text-xl text-gray-500">
                    Framework-provided validation rules for sync checks, derive attributes, and
                    async DB checks.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Rule Index</h2>
                <p>
                    Click <strong>Sample</strong> in any row to jump to runnable DTO examples.
                    Most samples below intentionally use the raw <code>validator</code> +{' '}
                    <code>schemars</code> style so you can see the underlying behavior directly.
                </p>

                <div className="not-prose overflow-x-auto">
                    <table className="min-w-full text-sm border-collapse border border-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="border border-gray-200 px-3 py-2 text-left">Type</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">Rule</th>
                                <th className="border border-gray-200 px-3 py-2 text-left">
                                    What It Does
                                </th>
                                <th className="border border-gray-200 px-3 py-2 text-left">
                                    Sample
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {RULE_ROWS.map((row) => (
                                <tr key={`${row.category}-${row.rule}`}>
                                    <td className="border border-gray-200 px-3 py-2">
                                        {row.category}
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <code>{row.rule}</code>
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">
                                        {row.description}
                                    </td>
                                    <td className="border border-gray-200 px-3 py-2">
                                        <button
                                            type="button"
                                            onClick={() => jumpToSample(row.sampleId)}
                                            className="rounded bg-orange-100 px-2 py-1 text-xs font-semibold text-orange-800 hover:bg-orange-200"
                                        >
                                            View {SAMPLE_TITLE[row.sampleId]}
                                        </button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <h2>Schemars and OpenAPI</h2>
                <p>
                    Runtime validation and OpenAPI schema are separate layers:
                </p>
                <ul>
                    <li>
                        Runtime: <code>validator</code> + <code>core_web::rules</code>
                    </li>
                    <li>
                        OpenAPI: <code>schemars::JsonSchema</code> + <code>#[schemars(...)]</code>
                    </li>
                </ul>
                <p>
                    Rustforge default DTO style is <code>#[rustforge_contract]</code> +{' '}
                    <code>#[rf(...)]</code>, which generates runtime validation attributes and
                    OpenAPI hints/extensions from one field-attribute style. Raw{' '}
                    <code>#[validate(...)]</code> + <code>#[schemars(...)]</code> remains the
                    manual fallback.
                </p>
                <p>
                    Project-specific reusable rules should use wrapper types (for example{' '}
                    <code>UsernameString</code>) as the single source of truth for runtime
                    validation + OpenAPI schema.
                </p>

                <h3>Rustforge Contract Macro (default)</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::contracts::rustforge_contract;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct AdminCreateInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(rule = "alpha_dash")]
    pub username: String,

    #[rf(email)]
    #[rf(length(min = 5, max = 120))]
    pub email: Option<String>,
}`}</code>
                </pre>
                <p>
                    OpenAPI will include schema constraints plus field-level{' '}
                    <code>x-rf-rules</code> metadata. Use <code>#[schemars(...)]</code> for
                    explicit overrides when needed.
                </p>

                <h2>Samples</h2>

                <h3 id="sample-required-trimmed" className="scroll-mt-24">
                    <code>required_trimmed</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_display_name(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value)
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ProfileInput {
    #[validate(custom(function = "validate_display_name"))]
    #[schemars(length(min = 1, max = 80))]
    pub display_name: String,
}`}</code>
                </pre>

                <h3 id="sample-alpha-dash" className="scroll-mt-24">
                    <code>alpha_dash</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_handle(value: &str) -> Result<(), ValidationError> {
    core_web::rules::alpha_dash(value)
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct HandleInput {
    #[validate(custom(function = "validate_handle"))]
    #[schemars(length(min = 3, max = 32), regex(pattern = "^[A-Za-z0-9_-]+$"))]
    pub handle: String,
}`}</code>
                </pre>

                <h3 id="sample-lowercase-slug" className="scroll-mt-24">
                    <code>lowercase_slug</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_slug(value: &str) -> Result<(), ValidationError> {
    core_web::rules::lowercase_slug(value)
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct SlugInput {
    #[validate(custom(function = "validate_slug"))]
    #[schemars(length(min = 3, max = 120), regex(pattern = "^[a-z0-9]+(?:-[a-z0-9]+)*$"))]
    pub slug: String,
}`}</code>
                </pre>

                <h3 id="sample-strong-password" className="scroll-mt-24">
                    <code>strong_password</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_password(value: &str) -> Result<(), ValidationError> {
    core_web::rules::strong_password(value)
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct PasswordInput {
    #[validate(custom(function = "validate_password"))]
    #[schemars(length(min = 8, max = 128), description = "Must include uppercase, lowercase, and number.")]
    pub password: String,
}`}</code>
                </pre>

                <h3 id="sample-sync-email" className="scroll-mt-24">
                    <code>email</code> (sync helper)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_contact_email(value: &str) -> Result<(), ValidationError> {
    core_web::rules::email(value)
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ContactInput {
    #[validate(custom(function = "validate_contact_email"))]
    #[schemars(format = "email", length(min = 5, max = 120))]
    pub email: String,
}`}</code>
                </pre>

                <h3 id="sample-one-of-none-of" className="scroll-mt-24">
                    <code>one_of</code> + <code>none_of</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_status(value: &str) -> Result<(), ValidationError> {
    core_web::rules::one_of(value, &["draft", "published", "archived"])
}

fn validate_username(value: &str) -> Result<(), ValidationError> {
    core_web::rules::none_of(value, &["root", "system", "admin"])
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct StatusInput {
    #[validate(custom(function = "validate_status"))]
    #[schemars(regex(pattern = "^(draft|published|archived)$"))]
    pub status: String,

    #[validate(custom(function = "validate_username"))]
    #[schemars(length(min = 3, max = 32))]
    pub username: String,
}`}</code>
                </pre>

                <h3 id="sample-comparison" className="scroll-mt-24">
                    <code>eq / gt / gte / lt / lte</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate, JsonSchema)]
#[validate(schema(function = "validate_amounts"))]
pub struct TransferInput {
    #[schemars(range(min = 1, max = 1_000_000))]
    pub min_amount: i64,
    #[schemars(range(min = 1, max = 1_000_000))]
    pub max_amount: i64,
}

fn validate_amounts(input: &TransferInput) -> Result<(), ValidationError> {
    core_web::rules::gte(&input.max_amount, &input.min_amount)?;
    core_web::rules::gt(&input.max_amount, &0)?;
    core_web::rules::lt(&input.min_amount, &1_000_001)?;
    core_web::rules::eq(&input.max_amount, &input.max_amount)?;
    Ok(())
}`}</code>
                </pre>

                <h3 id="sample-date-datetime" className="scroll-mt-24">
                    <code>date</code> + <code>datetime</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

fn validate_publish_date(value: &str) -> Result<(), ValidationError> {
    core_web::rules::date(value, "[year]-[month]-[day]")
}

fn validate_publish_at(value: &str) -> Result<(), ValidationError> {
    core_web::rules::datetime(value, "[year]-[month]-[day] [hour]:[minute]:[second]")
}

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct PublishInput {
    #[validate(custom(function = "validate_publish_date"))]
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2}$")]
    pub publish_date: String,

    #[validate(custom(function = "validate_publish_at"))]
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2} \\\\d{2}:\\\\d{2}:\\\\d{2}$")]
    pub publish_at: String,
}`}</code>
                </pre>

                <h3 id="sample-date-comparison" className="scroll-mt-24">
                    <code>date_*</code> + <code>datetime_*</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate, JsonSchema)]
#[validate(schema(function = "validate_window"))]
pub struct DateWindowInput {
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2}$")]
    pub start_date: String,
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2}$")]
    pub end_date: String,
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2} \\\\d{2}:\\\\d{2}:\\\\d{2}$")]
    pub start_at: String,
    #[schemars(pattern = "^\\\\d{4}-\\\\d{2}-\\\\d{2} \\\\d{2}:\\\\d{2}:\\\\d{2}$")]
    pub end_at: String,
}

fn validate_window(input: &DateWindowInput) -> Result<(), ValidationError> {
    core_web::rules::date_gte(
        &input.end_date,
        &input.start_date,
        "[year]-[month]-[day]",
    )?;
    core_web::rules::datetime_gt(
        &input.end_at,
        &input.start_at,
        "[year]-[month]-[day] [hour]:[minute]:[second]",
    )?;
    Ok(())
}`}</code>
                </pre>

                <h3 id="sample-derive-length" className="scroll-mt-24">
                    <code>length</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct UsernameInput {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 chars."))]
    #[schemars(length(min = 3, max = 32))]
    pub username: String,
}`}</code>
                </pre>

                <h3 id="sample-derive-range" className="scroll-mt-24">
                    <code>range</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RatingInput {
    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5."))]
    #[schemars(range(min = 1, max = 5))]
    pub rating: i32,
}`}</code>
                </pre>

                <h3 id="sample-derive-email" className="scroll-mt-24">
                    <code>email</code> (derive)
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RegisterEmailInput {
    #[validate(email(message = "Invalid email address."))]
    #[schemars(format = "email", length(min = 5, max = 120))]
    pub email: String,
}`}</code>
                </pre>

                <h3 id="sample-must-match" className="scroll-mt-24">
                    <code>must_match</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ResetPasswordInput {
    #[validate(length(min = 8))]
    #[schemars(length(min = 8, max = 128))]
    pub password: String,

    #[validate(must_match(other = "password", message = "Password confirmation mismatch."))]
    #[schemars(length(min = 8, max = 128))]
    pub password_confirmation: String,
}`}</code>
                </pre>

                <h3 id="sample-derive-phonenumber" className="scroll-mt-24">
                    <code>phonenumber(field = ...)</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ContactInput {
    #[validate(length(equal = 2))]
    #[schemars(length(min = 2, max = 2), description = "ISO2 country code, e.g. MY, US.")]
    pub contact_country_iso2: String,

    #[validate(phonenumber(field = contact_country_iso2, message = "Invalid phone for country."))]
    #[schemars(length(min = 6, max = 20), description = "Raw phone input, validated against country.")]
    pub contact_phone: String,
}`}</code>
                </pre>

                <h3 id="sample-async-validate" className="scroll-mt-24">
                    <code>AsyncValidate</code> with <code>Unique</code>, <code>Exists</code>,{' '}
                    <code>NotExists</code>, and <code>PhoneByCountryIso2</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::extract::AsyncValidate;
use core_web::rules::{AsyncRule, Exists, NotExists, PhoneByCountryIso2, Unique};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RegisterInput {
    #[validate(length(min = 3, max = 32))]
    #[schemars(length(min = 3, max = 32))]
    pub username: String,

    #[validate(email)]
    #[schemars(format = "email", length(min = 5, max = 120))]
    pub email: String,

    #[schemars(range(min = 1))]
    pub tenant_id: i64,

    #[validate(length(equal = 2))]
    #[schemars(length(min = 2, max = 2))]
    pub contact_country_iso2: String,

    #[validate(phonenumber(field = contact_country_iso2))]
    #[schemars(length(min = 6, max = 20))]
    pub contact_phone: String,
}

#[async_trait::async_trait]
impl AsyncValidate for RegisterInput {
    async fn validate_async(&self, db: &sqlx::PgPool) -> anyhow::Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if !Unique::new("users", "username", &self.username).check(db).await? {
            errors.add("username", ValidationError::new("unique"));
        }

        if !Exists::new("tenants", "id", &self.tenant_id).check(db).await? {
            errors.add("tenant_id", ValidationError::new("exists"));
        }

        if !NotExists::new("banned_users", "email", &self.email).check(db).await? {
            errors.add("email", ValidationError::new("not_exists"));
        }

        if !PhoneByCountryIso2::new(&self.contact_country_iso2, &self.contact_phone)
            .enabled_only(true)
            .check(db)
            .await?
        {
            errors.add("contact_phone", ValidationError::new("phone_country"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}`}</code>
                </pre>
            </div>
        </div>
    )
}
