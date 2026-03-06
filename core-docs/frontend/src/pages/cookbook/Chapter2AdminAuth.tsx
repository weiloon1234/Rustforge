import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter2AdminAuth() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Add Admin Auth and Permission Gates
                </h1>
                <p className="text-xl text-gray-500">
                    Build admin login/me/logout/password flows with typed guard identity and permission-aware routes.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Reuse scaffold auth architecture and keep auth logic in workflow layer with typed contracts.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Contracts: <code>app/src/contracts/api/v1/admin/auth.rs</code>
                    </li>
                    <li>
                        Routes: <code>app/src/internal/api/v1/admin/auth.rs</code>
                    </li>
                    <li>
                        Workflow: <code>app/src/internal/workflows/admin_auth.rs</code>
                    </li>
                    <li>
                        Guard middleware: <code>app/src/internal/middleware/auth.rs</code>
                    </li>
                </ul>

                <h3>Pattern summary</h3>
                <ul>
                    <li>Login endpoint returns session tokens and scopes snapshot.</li>
                    <li>
                        Frontend should hydrate the full account via <code>/me</code> immediately after login so
                        locale, admin type, and UI permission state come from the canonical account payload.
                    </li>
                    <li>
                        Protected endpoints use typed auth extractor (<code>AuthUser&lt;AdminGuard&gt;</code>).
                    </li>
                    <li>
                        Permission checks are added on business routes with{' '}
                        <code>with_permission_check_*</code> helpers.
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>Add refresh-token flow and device session model if product needs long-lived sessions.</li>
                    <li>Add tenant-bound scopes in workflow, not in handler.</li>
                    <li>
                        Add additional portal guards by extending permissions catalog and generated guard mapping.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./console route list
curl -X POST http://127.0.0.1:3000/api/v1/admin/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"password"}'
curl -H 'Authorization: Bearer <TOKEN>' http://127.0.0.1:3000/api/v1/admin/auth/me`}</code>
                </pre>
            </div>
        </div>
    )
}
