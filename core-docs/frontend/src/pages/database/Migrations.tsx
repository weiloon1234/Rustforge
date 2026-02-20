import { useEffect } from 'react'
import Prism from 'prismjs'

export function Migrations() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Migrations</h1>
                <p className="text-xl text-gray-500">
                    Database schema management via SQLx.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    We use <code>sqlx-cli</code> generally for migrations. However, we wrap it in our CLI
                    to provide a smoother experience and to enforce project structure.
                </p>

                <h2>Framework Migrations</h2>
                <p>
                    The framework automatically pumps (generates) a set of core migrations into your
                    <code>migrations/</code> folder when you run <code>make dev</code> or the migration command.
                    These provide the necessary schema for framework features.
                </p>

                <div className="bg-gray-50 border rounded-lg p-4">
                    <h3 className="mt-0 text-lg font-bold">Standard Framework Tables</h3>
                    <ul className="list-disc pl-5 mt-2 space-y-1 text-sm text-gray-700">
                        <li><code>0000000000001_meta.sql</code> — Dynamic metadata storage (JSONB)</li>
                        <li><code>0000000000002_attachments.sql</code> — File uploads and media tracking</li>
                        <li><code>0000000000003_localized.sql</code> — Internationalization (i18n) content</li>
                        <li><code>0000000000004_personal_access_tokens.sql</code> — API Tokens & Authentication</li>
                        <li><code>0000000000005_failed_jobs.sql</code> — Comparison queue for failed background jobs</li>
                        <li><code>0000000000006_outbox_jobs.sql</code> — Transactional outbox pattern</li>
                        <li><code>0000000000007_http_logs.sql</code> — Webhook and HTTP client logging</li>
                    </ul>
                </div>

                <h2>Managing Migrations</h2>

                <h3>Model PK with Default Snowflake IDs</h3>
                <p>
                    For application model tables using default <code>id_strategy = "snowflake"</code>{' '}
                    (<code>i64</code> PK), create PK columns as plain <code>BIGINT</code>.
                </p>
                <ul>
                    <li>Use: <code>BIGINT PRIMARY KEY</code></li>
                    <li>
                        Avoid: <code>BIGSERIAL</code> / <code>GENERATED AS IDENTITY</code> for
                        the same column
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-sql">{`CREATE TABLE articles (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);`}</code>
                </pre>

                <h3>Create a New Migration</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`# Using Make (Recommended)
make migrate-new name=create_posts

# Or using raw sqlx
sqlx migrate add create_posts`}</code>
                </pre>

                <h3>Run Migrations</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`# Using Make
make migrate

# Using CLI Wrapper
./console migrate run`}</code>
                </pre>

                <h3>Revert Last Migration</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`make migrate-revert`}</code>
                </pre>

                <h3>Check Status</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`make migrate-info`}</code>
                </pre>
            </div>
        </div>
    )
}
