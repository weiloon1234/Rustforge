export function OpenApi() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">OpenAPI</h1>
                <p className="text-xl text-gray-500">
                    Code-first API documentation that stays aligned with typed contracts, guards, and permissions.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What OpenAPI owns here</h2>
                <p>
                    OpenAPI is the machine-readable API contract surface. It should reflect the actual request,
                    response, guard, and permission model emitted by the framework rather than a second handwritten
                    spec.
                </p>

                <h2>Enable docs</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-env">{`ENABLE_OPENAPI_DOCS=true
OPENAPI_DOCS_PATH=/openapi
OPENAPI_JSON_PATH=/openapi.json`}</code>
                </pre>

                <h2>Route declaration pattern</h2>
                <p>
                    Declare authz once at the route. The same helper applies runtime checks and writes OpenAPI
                    metadata, so the docs and behavior do not drift.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::{ApiRouter, with_permission_check_post},
};
use generated::{guards::AdminGuard, permissions::Permission};

let router = ApiRouter::new().api_route(
    "/admin/content-pages",
    with_permission_check_post(
        create_content_page,
        AdminGuard,
        PermissionMode::Any,
        [Permission::ContentPageManage],
    ),
);`}</code>
                </pre>

                <h2>OpenAPI auth metadata</h2>
                <p>
                    Permission-aware helpers emit operation metadata that the console route list and external tools
                    can consume:
                </p>
                <ul>
                    <li><code>x-required-guard</code></li>
                    <li><code>x-required-permission-mode</code></li>
                    <li><code>x-required-permissions</code></li>
                </ul>

                <h2>Validation and schema generation</h2>
                <p>
                    Runtime validation and OpenAPI shape are related but different:
                </p>
                <ul>
                    <li>
                        Runtime validation comes from <code>#[rustforge_contract]</code>,{' '}
                        <code>#[rf(...)]</code>, validator traits, and optional async validation.
                    </li>
                    <li>
                        OpenAPI request/response schema comes from <code>JsonSchema</code> plus framework-added
                        hints such as <code>x-rf-rules</code>.
                    </li>
                </ul>
                <p>
                    The default path remains: use <code>#[rustforge_contract]</code> and let Rust own the DTO
                    shape. Do not hand-maintain matching OpenAPI-only DTO copies.
                </p>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
pub struct UpdateAdminInput {
    #[serde(default)]
    #[rf(email)]
    pub email: Patch<String>,
}`}</code>
                </pre>

                <h2>OpenAPI and generated TypeScript</h2>
                <p>
                    For the starter/frontend contract surface, the primary path is still <code>ts-rs</code> plus
                    <code>make gen-types</code>. OpenAPI remains the full external API description, not the main
                    source for app-local frontend types.
                </p>
                <ul>
                    <li>
                        Use <code>#[derive(TS)]</code> on contract-facing DTOs to export portal types from Rust.
                    </li>
                    <li>
                        Shared framework/platform types are exported into shared TS outputs from Rust-side sources.
                    </li>
                    <li>
                        Use OpenAPI-generated TypeScript only when you explicitly need a spec-driven client or a
                        monolithic external schema snapshot.
                    </li>
                </ul>

                <h2>Inspect the emitted API surface</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`./console route list
./console route list --json`}</code>
                </pre>
                <p>
                    Route list output includes guard and permission metadata parsed from the same OpenAPI extensions.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/requests">Requests &amp; Validation</a> for contract boundary rules.
                    </li>
                    <li>
                        <a href="#/auth">Guards &amp; Auth</a> for session lifecycle and account hydration.
                    </li>
                    <li>
                        <a href="#/permissions">Permissions &amp; AuthZ</a> for matcher and delegation semantics.
                    </li>
                </ul>
            </div>
        </div>
    )
}
