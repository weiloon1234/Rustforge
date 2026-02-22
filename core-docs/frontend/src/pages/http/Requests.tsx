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
use core_web::extract::AsyncValidatedJson;

#[rustforge_contract]
pub struct RegisterInput {
    #[rf(length(min = 3, max = 32))]
    #[rf(alpha_dash)]
    pub username: String,
    #[rf(async_unique(table = "admin", column = "username"))]
    pub login_username: String,
}

async fn register(
    AsyncValidatedJson(req): AsyncValidatedJson<RegisterInput>,
) -> Result<ApiResponse<()>, AppError> {
    let _ = req;
    Ok(ApiResponse::success((), "ok"))
}`}</code>
                </pre>
                <p>
                    Rustforge can generate <code>AsyncValidate</code> automatically for simple DB
                    checks with <code>rf(async_unique)</code>, <code>rf(async_exists)</code>, and{' '}
                    <code>rf(async_not_exists)</code>. For complex multi-field DB logic, keep a
                    manual <code>impl AsyncValidate</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
pub struct AdminUpdateInput {
    pub id: i64,
    pub tenant_id: i64,

    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "id"),
        where_eq(column = "tenant_id", field = "tenant_id"),
        where_null(column = "deleted_at")
    ))]
    pub username: String,
}`}</code>
                </pre>
                <p>
                    For <code>PATCH /resource/{'{id}'}</code> updates, the ignore key often comes
                    from the path, not the JSON body. Keep the path parameter as the source of
                    truth, inject it into a hidden DTO field, then run async validation once.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
pub struct UpdateAdminInput {
    #[serde(skip, default)]
    __target_id: i64,

    #[serde(default)]
    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "__target_id")
    ))]
    pub username: Option<String>,
}

impl UpdateAdminInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}`}</code>
                </pre>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    contracts::{AsyncContractJson, ContractJson},
    error::AppError,
    extract::{validation::transform_validation_errors, AsyncValidate},
};

async fn create(
    State(state): State<AppApiState>,
    req: AsyncContractJson<CreateAdminInput>,
) -> Result<(), AppError> {
    let _req = req.0; // async rf(...) already executed
    let _ = state;
    Ok(())
}

async fn update(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
    req: ContractJson<UpdateAdminInput>,
) -> Result<(), AppError> {
    let req = req.0.with_target_id(id);
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    // req is trusted here
    Ok(())
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
                        <code>#[rf(openapi(example = ...))]</code> accepts literals/expressions (for
                        example numbers and booleans), not only strings.
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
pub struct ExampleInput {
    #[rf(range(min = 1))]
    pub owner_id: i64,

    #[rf(length(min = 1, max = 64))]
    #[rf(required_trimmed)]
    pub title: String,
}`}</code>
                </pre>
            </div>
        </div>
    )
}
