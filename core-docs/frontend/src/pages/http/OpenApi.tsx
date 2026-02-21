export function OpenApi() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">OpenAPI Documentation</h1>
                <p className="text-xl text-gray-500">
                    Aide code-first docs with machine-readable authz metadata.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Overview</h2>
                <p>
                    Route signatures generate schemas automatically from extractor/request/response
                    types. For protected routes, add permission declaration once via{' '}
                    <code>with_permission_check_*</code> helpers so runtime checks and OpenAPI stay
                    in sync.
                </p>

                <h2>Enable Docs</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-env">{`ENABLE_OPENAPI_DOCS=true
OPENAPI_DOCS_PATH=/openapi
OPENAPI_JSON_PATH=/openapi.json`}</code>
                </pre>
                <p>
                    Guard is inferred from <code>AdminGuard::name()</code>; no duplicated guard
                    string in route declarations.
                </p>
                <p>
                    When enabled: <code>/openapi</code> serves Redoc and{' '}
                    <code>/openapi.json</code> serves the spec.
                </p>

                <h2>Canonical Protected Route Helper</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::{with_permission_check_post, with_bearer_auth_scheme},
};
use generated::{guards::AdminGuard, permissions::Permission};

let api_router = ApiRouter::new().api_route(
    "/admin/articles",
    with_permission_check_post(
        create_article,
        AdminGuard,
        PermissionMode::Any,
        [Permission::ArticleManage],
    ),
);`}</code>
                </pre>

                <h2>Operation Extensions</h2>
                <p>The helper writes these operation-level extensions:</p>
                <ul>
                    <li>
                        <code>x-required-guard</code>
                    </li>
                    <li>
                        <code>x-required-permission-mode</code>
                    </li>
                    <li>
                        <code>x-required-permissions</code>
                    </li>
                </ul>
                <p>
                    It also adds bearer security + machine-readable authz metadata, and applies
                    runtime permission middleware on the route.
                </p>

                <h2>Validation vs OpenAPI Constraints</h2>
                <p>
                    Runtime validation and OpenAPI constraints are related but not identical:
                </p>
                <ul>
                    <li>
                        Runtime checks come from <code>validator</code> attributes and optional async
                        validation.
                    </li>
                    <li>
                        OpenAPI request constraints come from <code>JsonSchema</code> and{' '}
                        <code>#[schemars(...)]</code> annotations.
                    </li>
                </ul>
                <p>
                    <strong>Important:</strong> <code>#[validate(...)]</code> alone does not
                    auto-write OpenAPI length/range constraints. Add matching{' '}
                    <code>#[schemars(...)]</code> rules explicitly.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[derive(Debug, Deserialize, Validate, JsonSchema)]
pub struct ArticleCreateInput {
    #[validate(range(min = 1))]
    #[schemars(range(min = 1))]
    pub category_id: i64,

    #[validate(length(min = 1, max = 32))]
    #[schemars(length(min = 1, max = 32))]
    pub status: generated::models::ArticleStatus,

    pub title: generated::localized::MultiLang,
    pub summary: generated::localized::MultiLang,

    pub cover: Option<core_db::platform::attachments::types::AttachmentUploadDto>,
    pub galleries: Vec<core_db::platform::attachments::types::AttachmentUploadDto>,
}`}</code>
                </pre>
                <p>
                    With this shape, OpenAPI can display enums, nested objects, and numeric/string
                    constraints in request schemas.
                </p>

                <h2>Route Introspection</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`./console route list
./console route list --json`}</code>
                </pre>
                <p>
                    Route list output now includes guard, permission mode, and permission keys
                    parsed from OpenAPI extensions.
                </p>

                <h2>TypeScript Generation (OpenAPI-first)</h2>
                <p>
                    Generate TypeScript client/types from <code>/openapi.json</code> as the single
                    source for API contracts:
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`curl -sS http://127.0.0.1:3000/openapi.json > openapi.json
npx openapi-typescript openapi.json -o src/types/openapi.d.ts`}</code>
                </pre>
            </div>
        </div>
    )
}
