export function Permissions() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Permissions &amp; AuthZ</h1>
                <p className="text-xl text-gray-500">
                    Typed permission catalog with PAT-scope runtime enforcement.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>1) Catalog SSOT</h2>
                <p>
                    Define permissions in <code>app/permissions.toml</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[[permissions]]
key = "article.manage"
guard = "admin"
label = "Manage Articles"
group = "article"
description = "Create, update, and delete articles."`}</code>
                </pre>

                <h2>2) Generated Typed API</h2>
                <p>
                    <code>cargo check -p generated</code> generates{' '}
                    <code>generated::permissions</code>:
                </p>
                <ul>
                    <li>
                        <code>Permission</code> enum
                    </li>
                    <li>
                        <code>PermissionMeta</code> metadata
                    </li>
                    <li>
                        helpers: <code>as_str</code>, <code>from_str</code>, <code>all</code>,{' '}
                        <code>by_guard</code>
                    </li>
                </ul>

                <h2>3) Runtime Source of Truth</h2>
                <p>
                    Runtime checks read scopes from <code>personal_access_tokens.abilities</code>{' '}
                    only. There is no per-request permission-table join.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::auth::TokenScopeGrant;

let scope = match admin.admin_type {
    AdminType::Developer | AdminType::SuperAdmin => TokenScopeGrant::Wildcard,
    AdminType::Admin => TokenScopeGrant::AuthOnly,
};`}</code>
                </pre>

                <h2>4) AuthZ Helpers</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Route-level (runtime + OpenAPI):
core_web::openapi::with_permission_check_post(
    create,
    generated::guards::AdminGuard,
    core_web::authz::PermissionMode::Any,
    [generated::permissions::Permission::ArticleManage],
);

core_web::openapi::with_permission_check_get(
    export_csv,
    generated::guards::AdminGuard,
    core_web::authz::PermissionMode::Any,
    [generated::permissions::Permission::ArticleExport],
);

// Workflow-level:
core_web::authz::ensure_permissions(
    &auth,
    core_web::authz::PermissionMode::Any,
    &[generated::permissions::Permission::ArticleManage],
)?;`}</code>
                </pre>
                <p>
                    Convention: <code>resource.manage</code> implicitly grants{' '}
                    <code>resource.read</code>. Keep specialized actions separate (for example{' '}
                    <code>article.export</code>).
                </p>

                <h2>5) OpenAPI Contract</h2>
                <p>
                    Use <code>with_permission_check_*</code> helpers so OpenAPI includes authz
                    metadata: <code>x-required-guard</code>,{' '}
                    <code>x-required-permission-mode</code>, and{' '}
                    <code>x-required-permissions</code>.
                </p>
            </div>
        </div>
    )
}
