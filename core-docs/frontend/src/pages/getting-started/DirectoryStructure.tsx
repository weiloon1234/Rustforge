export function DirectoryStructure() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Directory Structure</h1>
                <p className="text-xl text-gray-500">
                    Rustforge is framework-only. Your app lives in a separate starter repository.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h3 className="text-lg font-bold text-gray-900 mb-2">Rustforge Repository</h3>
                <p className="text-sm text-gray-600">
                    Top-level crates only. No <code>apps/</code> folder in framework workspace.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-text">{`Rustforge/
├── Cargo.toml
├── bootstrap/
├── core-config/
├── core-db/
├── core-datatable/
├── core-docs/
├── core-http-log/
├── core-i18n/
├── core-jobs/
├── core-mailer/
├── core-notify/
├── core-realtime/
├── core-web/
├── db-gen/
├── scaffold/
└── vendor/`}</code>
                </pre>

                <h3 className="text-lg font-bold text-gray-900 mt-8 mb-2">Starter Repository</h3>
                <p className="text-sm text-gray-600">
                    Scaffold generates this compile-first app layout.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-text">{`starter/
├── Cargo.toml
├── .env.example
├── Makefile
├── i18n/
├── bin/
├── migrations/
├── app/
│   ├── configs.toml
│   ├── permissions.toml
│   ├── schemas/
│   └── src/
│       ├── bin/
│       ├── contracts/
│       ├── validation/
│       ├── seeds/
│       └── internal/
└── generated/`}</code>
                </pre>

                <h3 className="text-lg font-bold text-gray-900 mt-8 mb-2">Single Source of Truth</h3>
                <ul className="list-disc pl-5 space-y-2 text-gray-700">
                    <li>
                        Schema: <code>app/schemas/*.toml</code>
                    </li>
                    <li>
                        Permissions: <code>app/permissions.toml</code>
                    </li>
                    <li>
                        App config: <code>app/configs.toml</code>
                    </li>
                    <li>
                        Generated output: <code>generated/src/*</code>
                    </li>
                </ul>
            </div>
        </div>
    )
}
