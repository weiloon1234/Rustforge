import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter1CrudApiServer() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Build a CRUD Admin Resource
                </h1>
                <p className="text-xl text-gray-500">
                    Start from scaffold admin domain, then extend to new CRUD modules without breaking SSOT.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Build CRUD routes with generated models, typed contracts, and permission-aware OpenAPI using
                    scaffold conventions.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Model baseline: <code>app/models/admin.rs</code>
                    </li>
                    <li>
                        DTO contracts: <code>app/src/contracts/api/v1/admin/*.rs</code>
                    </li>
                    <li>
                        API handlers: <code>app/src/internal/api/v1/admin/*.rs</code>
                    </li>
                    <li>
                        Workflow layer: <code>app/src/internal/workflows/admin.rs</code>
                    </li>
                    <li>
                        Portal composer: <code>app/src/internal/api/v1/mod.rs</code>
                    </li>
                    <li>
                        Typed permissions: <code>generated::permissions::Permission</code> from{' '}
                        <code>app/permissions.toml</code>
                    </li>
                </ul>

                <h3>Recommended baseline route style</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::{with_permission_check_get, with_permission_check_post, ApiRouter},
};
use generated::{guards::AdminGuard, permissions::Permission};

pub fn routes(state: AppApiState) -> ApiRouter<AppApiState> {
    ApiRouter::new()
        .api_route(
            "/accounts",
            with_permission_check_get(index, AdminGuard, PermissionMode::Any, [Permission::AdminRead])
                .merge(with_permission_check_post(
                    create,
                    AdminGuard,
                    PermissionMode::Any,
                    [Permission::AdminManage],
                )),
        )
        .with_state(state)
}`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <p>
                    For a new domain (example article/category), keep the same layering and only add new modules.
                </p>
                <ul>
                    <li>Add model file(s) under <code>app/models/*.rs</code>.</li>
                    <li>Add migration SQL under <code>migrations/</code>.</li>
                    <li>Add DTOs under <code>app/src/contracts/api/v1/&lt;portal&gt;/</code>.</li>
                    <li>
                        Add handlers under <code>app/src/internal/api/v1/&lt;portal&gt;/</code> and compose in{' '}
                        <code>app/src/internal/api/v1/mod.rs</code>.
                    </li>
                    <li>
                        Add workflow logic under <code>app/src/internal/workflows/</code> and keep handlers thin.
                    </li>
                </ul>

                <h2>Computed Model Value (Scaffold Now)</h2>
                <p>
                    Put computed/read-only fields on <code>AdminView</code> methods in the model source file, not on raw
                    <code> AdminRow</code>.
                </p>
                <ul>
                    <li>
                        Extension point: <code>app/models/admin.rs</code> with <code>#[rf_record_impl]</code>
                    </li>
                    <li>
                        API contract mapping: <code>app/src/contracts/api/v1/admin/*.rs</code>
                    </li>
                    <li>
                        Optional datatable row injection: <code>app/src/internal/datatables/v1/admin/account.rs</code>
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`impl From<generated::models::AdminView> for AdminOutput {
    fn from(value: generated::models::AdminView) -> Self {
        Self {
            id: value.id,
            identity: value.identity(), // username -> name -> email fallback
            username: value.username,
            email: value.email,
            name: value.name,
            admin_type: value.admin_type,
            abilities: vec![],
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}`}</code>
                </pre>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo check -p app
./console migrate run
./console route list --json
curl -sS http://127.0.0.1:3000/openapi.json | jq '.paths | keys[]'`}</code>
                </pre>
            </div>
        </div>
    )
}
