import { useEffect } from 'react'
import Prism from 'prismjs'

export function CookbookOverview() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Cookbook</h1>
                <p className="text-xl text-gray-500">
                    Practical app-level recipes using framework crates with single-source-of-truth
                    conventions.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Core Rules</h2>
                <ul>
                    <li>
                        Schema SSOT: <code>app/schemas/*.toml</code>.
                    </li>
                    <li>
                        Permission SSOT: <code>app/permissions.toml</code>.
                    </li>
                    <li>
                        DTO SSOT: <code>app/src/contracts/api/*</code>.
                    </li>
                    <li>
                        Never edit generated files manually under <code>generated/*</code>.
                    </li>
                    <li>
                        API paths use portal prefixes under <code>/api/v1/user/*</code> and{' '}
                        <code>/api/v1/admin/*</code>.
                    </li>
                    <li>
                        User-facing API text should use <code>core_i18n::t("...")</code>.
                    </li>
                    <li>
                        Input DTO default style: <code>#[rustforge_contract]</code> +{' '}
                        <code>#[rf(...)]</code> (raw <code>#[validate(...)]</code> +{' '}
                        <code>#[schemars(...)]</code> is the escape hatch).
                    </li>
                    <li>
                        Prefer <code>core_web::contracts::ContractJson&lt;T&gt;</code> on handlers
                        and keep <code>T</code> under the <code>RequestContract</code> standard.
                    </li>
                    <li>
                        Use the standalone <code>#/validation-rules</code> guide for framework
                        validation rules + <code>schemars</code> usage.
                    </li>
                </ul>

                <h2>Portal Routing Standard</h2>
                <ul>
                    <li>
                        Keep portal prefixes centralized in <code>app/src/api/v1/mod.rs</code>.
                    </li>
                    <li>
                        Module files export route collections with relative paths: <code>user_routes()</code>,{' '}
                        <code>user_auth_routes(state)</code>, <code>admin_routes(state)</code>.
                    </li>
                    <li>
                        Use guard middleware for auth boundary and workflow/helper checks for business authz.
                    </li>
                </ul>

                <h2>Admin AuthZ Standard</h2>
                <ul>
                    <li>
                        Define catalog in <code>app/permissions.toml</code>.
                    </li>
                    <li>
                        Use generated enum <code>generated::permissions::Permission</code>, not free
                        strings.
                    </li>
                    <li>
                        Runtime checks use <code>core_web::authz</code> with default mode{' '}
                        <code>PermissionMode::Any</code>.
                    </li>
                    <li>
                        Use <code>core_web::openapi::with_permission_check_*</code> helpers so OpenAPI
                        metadata and runtime permission checks come from one declaration.
                    </li>
                    <li>
                        Prefer <code>read/manage</code> action model. <code>resource.manage</code>{' '}
                        implicitly grants <code>resource.read</code>.
                    </li>
                    <li>
                        For actor-type policies (example developer-only route), use app-level model
                        extension helpers (scaffold: <code>models::admin_ext::AdminViewExt</code>).
                    </li>
                    <li>
                        Route introspection uses <code>./console route list</code> to read required guard and
                        permissions from <code>x-required-*</code> extensions.
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_web::{
    authz::PermissionMode,
    openapi::with_permission_check_post,
};
use generated::guards::AdminGuard;
use generated::permissions::Permission;

with_permission_check_post(
    create,
    AdminGuard,
    PermissionMode::Any,
    [Permission::ArticleManage],
)`}</code>
                </pre>

                <h2>Chapter Format</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-markdown">{`1) create schema file(s)
2) create migration(s)
3) define/refresh permission catalog (if protected routes exist)
4) regenerate generated APIs/types
5) create/update DTOs in app/src/contracts
6) choose handler style (thin handler + workflow, or fat handler)
7) add model extensions (default: XxxView, optional: XxxQuery)
8) use with_permission_check_* helpers for runtime + OpenAPI authz
9) verify with ./console route list and curl`}</code>
                </pre>
                <p>
                    TypeScript generation standard is OpenAPI-first from{' '}
                    <code>/openapi.json</code>.
                </p>

                <h2>Recommended Reading Order (No Renumber Needed)</h2>
                <p>
                    Current chapter numbering is still valid. For new teams, the clearest path is
                    to read by dependency order instead of renumbering the cookbook again.
                </p>
                <ol>
                    <li>
                        Foundation docs: <a href="#/requests">Requests</a>,{' '}
                        <a href="#/validation-rules">Validation Rules</a>,{' '}
                        <a href="#/openapi">OpenAPI</a>, <a href="#/permissions">Permissions &amp; AuthZ</a>.
                    </li>
                    <li>Chapter 1 (CRUD API baseline + portal routing + DTO pattern).</li>
                    <li>Chapter 2A (DTO + validation rules + wrapper types + OpenAPI alignment).</li>
                    <li>Chapter 2B (Admin auth + token scopes + admin CRUD async validation patterns).</li>
                    <li>Chapter 7 (CLI workflow) if the team is setting up daily commands early.</li>
                    <li>Chapters 3-6 for async/jobs/notifications/realtime features.</li>
                    <li>Chapter 8 then Chapter 9 for integration and hardening.</li>
                    <li>Chapters 10-12 for caching, testing, and event fan-out patterns.</li>
                </ol>
                <p>
                    Chapter numbering for 3-9 stays unchanged to avoid link churn. Chapter 2 is
                    split as 2A/2B only.
                </p>

                <h2>Available Chapters</h2>
                <div className="not-prose grid grid-cols-1 gap-3">
                    <a
                        href="#/cookbook-chapter-1-crud-api-server"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 1: CRUD API (portal routing + typed permissions)
                    </a>
                    <a
                        href="#/cookbook-chapter-2-validation-dto"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 2A: Request/Response DTO + Validation
                    </a>
                    <a
                        href="#/cookbook-chapter-2-admin-auth"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 2B: Admin Auth + DTO Async Validation Patterns
                    </a>
                    <a
                        href="#/cookbook-chapter-3-jobs-usage"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 3: Jobs Usage
                    </a>
                    <a
                        href="#/cookbook-chapter-4-notifications"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 4: Notifications
                    </a>
                    <a
                        href="#/cookbook-chapter-5-websocket-channel"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 5: WebSocket Channel
                    </a>
                    <a
                        href="#/cookbook-chapter-6-websocket-auth-middleware"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 6: WebSocket Auth
                    </a>
                    <a
                        href="#/cookbook-chapter-7-cli-workflow"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 7: CLI Workflow
                    </a>
                    <a
                        href="#/cookbook-chapter-8-end-to-end-flow"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 8: End-to-End
                    </a>
                    <a
                        href="#/cookbook-chapter-9-production-hardening"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 9: Hardening
                    </a>
                    <a
                        href="#/cookbook-chapter-10-caching"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 10: Caching Recipe
                    </a>
                    <a
                        href="#/cookbook-chapter-11-testing"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 11: Testing Recipe
                    </a>
                    <a
                        href="#/cookbook-chapter-12-event-fanout"
                        className="px-4 py-3 border border-gray-200 rounded-md bg-white hover:bg-gray-50 text-sm font-medium text-gray-800"
                    >
                        Chapter 12: Event Fan-Out Recipe
                    </a>
                </div>
            </div>
        </div>
    )
}
