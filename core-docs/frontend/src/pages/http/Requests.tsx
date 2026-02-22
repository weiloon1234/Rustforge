export function Requests() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Requests</h1>
                <p className="text-xl text-gray-500">
                    Extractors, validation boundary, and OpenAPI schema alignment.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    For full Laravel-style rule mapping and examples, see{' '}
                    <a href="#/validation-rules">Validation Rules</a>.
                </p>

                <h2>Extractors</h2>
                <p>
                    Use <code>core_web::contracts</code> + <code>core_web::extract</code> as the
                    canonical request boundary surface:
                </p>
                <ul className="list-disc pl-5">
                    <li>
                        <code>RequestContract</code>: DTO contract trait (
                        <code>Deserialize + Validate + JsonSchema</code>).
                    </li>
                    <li>
                        <code>ContractJson&lt;T&gt;</code>: ergonomic alias of{' '}
                        <code>ValidatedJson&lt;T&gt;</code>.
                    </li>
                    <li>
                        <code>ValidatedJson&lt;T&gt;</code>: JSON parse + sync{' '}
                        <code>validator::Validate</code>.
                    </li>
                    <li>
                        <code>AsyncValidatedJson&lt;T&gt;</code>: sync + async DB rules via{' '}
                        <code>AsyncValidate</code>.
                    </li>
                </ul>

                <h2>Trusted Boundary</h2>
                <p>
                    Once extraction succeeds, <code>req</code> is trusted validated input. Avoid
                    re-validating the same fields in workflow/handler.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_web::contracts::ContractJson;

async fn create(
    ContractJson(req): ContractJson<MyCreateInput>,
) -> Result<ApiResponse<MyOutput>, AppError> {
    // req is already validated here
    run_workflow(req).await
}`}</code>
                </pre>

                <h2>Sync + Async Validation</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::contracts::rustforge_contract;
use core_web::extract::{AsyncValidate, AsyncValidatedJson};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::{Validate, ValidationErrors};

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct RegisterInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(rule = "alpha_dash")]
    pub username: String,
}

#[async_trait::async_trait]
impl AsyncValidate for RegisterInput {
    async fn validate_async(&self, db: &sqlx::PgPool) -> anyhow::Result<(), ValidationErrors> {
        // optional DB-backed checks
        let _ = db;
        Ok(())
    }
}

async fn register(
    AsyncValidatedJson(req): AsyncValidatedJson<RegisterInput>,
) -> Result<ApiResponse<()>, AppError> {
    let _ = req;
    Ok(ApiResponse::success((), "ok"))
}`}</code>
                </pre>

                <h2>OpenAPI Mapping Rules</h2>
                <p>
                    OpenAPI request schema is generated from <code>JsonSchema</code>. Runtime
                    validation runs from <code>validator</code>. Rustforge default DTO style is
                    <code>#[rustforge_contract]</code> + <code>#[rf(...)]</code>.
                </p>
                <ul className="list-disc pl-5">
                    <li>
                        Default: use <code>#[rf(...)]</code> on fields and let the macro emit
                        runtime + OpenAPI hints.
                    </li>
                    <li>
                        Fallback/escape hatch: use raw <code>#[validate(...)]</code> and{' '}
                        <code>#[schemars(...)]</code> manually.
                    </li>
                    <li>
                        For enum dropdown/options: use enum types that derive{' '}
                        <code>JsonSchema</code>.
                    </li>
                </ul>
                <p>
                    <strong>Important:</strong> raw <code>#[validate(...)]</code> does not always
                    fully describe OpenAPI constraints. If you skip <code>#[rf(...)]</code>, add
                    matching <code>#[schemars(...)]</code> hints explicitly.
                </p>

                <h2>Common Patterns</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::contracts::rustforge_contract;

#[rustforge_contract]
#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ExampleInput {
    #[rf(range(min = 1))]
    pub owner_id: i64,

    #[rf(length(min = 1, max = 64))]
    #[rf(rule = "required_trimmed")]
    pub title: String,
}`}</code>
                </pre>
            </div>
        </div>
    )
}
