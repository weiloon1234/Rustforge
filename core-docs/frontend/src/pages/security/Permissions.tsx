export function Permissions() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Permissions &amp; AuthZ</h1>
                <p className="text-xl text-gray-500">
                    Typed permission catalog, runtime PAT scope matching, and explicit delegation rules across backend and frontend.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Permission SSOT</h2>
                <p>
                    Define the catalog once in <code>app/permissions.toml</code>. That file remains the source of truth for permission keys, labels, grouping, and generated typed enums.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[[permissions]]
key = "content_page.manage"
guard = "admin"
label = "Manage Content Pages"
group = "content_page"`}</code>
                </pre>

                <h2>Generated typed surface</h2>
                <ul>
                    <li>Rust: <code>generated::permissions::Permission</code> and metadata helpers.</li>
                    <li>TypeScript: generated permission union/object from <code>make gen-types</code>.</li>
                    <li>Route metadata: permission declarations flow into OpenAPI via <code>with_permission_check_*</code>.</li>
                </ul>

                <h2>Runtime matching rules</h2>
                <p>
                    Runtime checks read token abilities from <code>personal_access_tokens.abilities</code>. There is no request-time permission-table join.
                </p>
                <table>
                    <thead>
                        <tr>
                            <th>Granted scope</th>
                            <th>Required scope</th>
                            <th>Result</th>
                            <th>Why</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><code>*</code></td>
                            <td><code>country.manage</code></td>
                            <td>allow</td>
                            <td>Global wildcard.</td>
                        </tr>
                        <tr>
                            <td><code>country.*</code></td>
                            <td><code>country.read</code></td>
                            <td>allow</td>
                            <td>Resource wildcard.</td>
                        </tr>
                        <tr>
                            <td><code>country.manage</code></td>
                            <td><code>country.read</code></td>
                            <td>allow</td>
                            <td><code>manage -&gt; read</code> convention.</td>
                        </tr>
                        <tr>
                            <td><code>country.read</code></td>
                            <td><code>country.manage</code></td>
                            <td>deny</td>
                            <td>No reverse implication.</td>
                        </tr>
                        <tr>
                            <td><code>export</code></td>
                            <td><code>content_page.read</code></td>
                            <td>deny</td>
                            <td>Standalone permissions do not imply resource permissions.</td>
                        </tr>
                        <tr>
                            <td><code>content_page.manage</code></td>
                            <td><code>country.read</code></td>
                            <td>deny</td>
                            <td>Implication only applies within the same resource scope.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Practical permission rules</h2>
                <ul>
                    <li><code>resource.manage</code> implies <code>resource.read</code>.</li>
                    <li>Standalone permissions such as <code>export</code> do not get special implication rules.</li>
                    <li>Wildcard scopes still work because stored abilities remain raw strings.</li>
                    <li>App-level exceptions stay app policy, not global matcher behavior.</li>
                </ul>

                <h2>Backend typed checks</h2>
                <p>
                    Prefer typed checks in handlers, workflows, and model extensions instead of raw string compares.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_web::authz::PermissionMode;
use generated::permissions::Permission;

core_web::openapi::with_permission_check_post(
    create,
    generated::guards::AdminGuard,
    PermissionMode::Any,
    [Permission::ContentPageManage],
);

core_web::authz::ensure_permissions(
    &auth,
    PermissionMode::Any,
    &[Permission::ContentPageManage],
)?;`}</code>
                </pre>
                <p>
                    For app-facing helpers, add typed permission methods in <code>app/models/admin.rs</code> via <code>#[rf_view_impl]</code> so workflows can call <code>auth.user.has_permission(...)</code> instead of scattering matcher details.
                </p>

                <h2>Frontend typed checks</h2>
                <p>
                    Keep auth payload scopes as raw <code>string[]</code> because wildcard matching works on the stored scope strings, but expose typed helpers in the portal auth store so UI gates read like the backend.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-ts">{`import { PERMISSION } from '@admin/types/enums'
import { useAuthStore } from '@admin/stores/auth'

const canManage = useAuthStore((s) =>
  useAuthStore.hasPermission(PERMISSION.content_page_manage, s.account)
)`}</code>
                </pre>

                <h2>Delegation policy matrix</h2>
                <p>
                    Assigning permissions to other admins is a business rule layered on top of the generic matcher. The starter should filter assignable permissions in the frontend and enforce the same rule again in workflows.
                </p>
                <table>
                    <thead>
                        <tr>
                            <th>Actor type</th>
                            <th>Can pass permission gate</th>
                            <th>Can assign only owned permissions</th>
                            <th>Special restrictions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><code>developer</code></td>
                            <td>yes</td>
                            <td>no</td>
                            <td>Framework/starter treat developer as all-permissions.</td>
                        </tr>
                        <tr>
                            <td><code>superadmin</code></td>
                            <td>yes</td>
                            <td>no</td>
                            <td>Starter treats superadmin as all-permissions for admin gating.</td>
                        </tr>
                        <tr>
                            <td><code>admin</code></td>
                            <td>only if owned or implied</td>
                            <td>yes</td>
                            <td>Starter additionally blocks delegating <code>admin.read</code> and <code>admin.manage</code>.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Starter admin-table rules</h2>
                <ul>
                    <li>Normal admins can only assign permissions they themselves have.</li>
                    <li>Normal admins cannot assign <code>admin.read</code> or <code>admin.manage</code>, even if they personally have them.</li>
                    <li>Normal admins still benefit from <code>manage -&gt; read</code>, so owning <code>content_page.manage</code> allows assigning <code>content_page.read</code>.</li>
                    <li>Self-edit and self-delete from the admin CRUD table are blocked by starter policy.</li>
                </ul>

                <h2>Related docs</h2>
                <ul>
                    <li><a href="#/permission-matrix">Permission Matrix</a> for the short operational cheat sheet.</li>
                    <li><a href="#/auth">Guards &amp; Auth</a> for token issue/refresh and guard wiring.</li>
                    <li><a href="#/openapi">OpenAPI</a> for route metadata emitted by permission-aware helpers.</li>
                    <li><a href="#/cookbook/add-admin-auth-permission-gates">Add Admin Auth &amp; Permission Gates</a> for the starter auth recipe.</li>
                </ul>
            </div>
        </div>
    )
}
