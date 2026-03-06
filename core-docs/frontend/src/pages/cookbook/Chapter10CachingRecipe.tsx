import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter10CachingRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Add Caching
                </h1>
                <p className="text-xl text-gray-500">
                    Cache read-heavy runtime data without moving source-of-truth away from the database and workflows.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Core rule</h2>
                <p>
                    Database and workflow logic stay authoritative. Cache is a read-optimization layer only. Do not move validation, write semantics, or business truth into cache.
                </p>

                <h2>Where caching belongs</h2>
                <ul>
                    <li>Read paths: workflow or API-layer helpers that assemble repeated runtime payloads</li>
                    <li>Invalidation: workflow write paths only</li>
                    <li>Key naming: stable constants owned by the feature module</li>
                </ul>

                <h2>Scaffold example: bootstrap countries</h2>
                <p>
                    The scaffold country runtime already demonstrates the intended pattern: cache the enabled bootstrap country list, invalidate after status changes, and keep the DB query as the underlying source.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub const BOOTSTRAP_COUNTRIES_CACHE_KEY: &str = "bootstrap:countries:enabled:v1";
pub const BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS: u64 = 300;

pub async fn list_enabled_for_bootstrap(state: &AppApiState) -> Result<Vec<Country>, AppError> {
    state.redis.remember(
        BOOTSTRAP_COUNTRIES_CACHE_KEY,
        BOOTSTRAP_COUNTRIES_CACHE_TTL_SECS,
        move || async move {
            // DB query remains source of truth
            /* ... */
        },
    ).await
}

pub async fn invalidate_bootstrap_country_cache(state: &AppApiState) -> Result<(), AppError> {
    state.redis.forget(BOOTSTRAP_COUNTRIES_CACHE_KEY).await?;
    Ok(())
}`}</code>
                </pre>

                <h2>Cache key design</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Good key</th>
                            <th>Why</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><code>bootstrap:countries:enabled:v1</code></td>
                            <td>clear feature ownership, query intent, and version.</td>
                        </tr>
                        <tr>
                            <td><code>dashboard:admin:summary:v1:{account_id}</code></td>
                            <td>scoped by actor when data visibility depends on actor.</td>
                        </tr>
                        <tr>
                            <td><code>content-page:by-tag:v1:{tag}</code></td>
                            <td>stable lookup key for repeated reads.</td>
                        </tr>
                    </tbody>
                </table>
                <p>
                    Include actor or locale in the key when those change the visible result. Include a version segment when the payload shape or query meaning might change.
                </p>

                <h2>What to cache well</h2>
                <ul>
                    <li>bootstrap runtime payload fragments</li>
                    <li>expensive summary cards and aggregate endpoints</li>
                    <li>stable lookup/reference data</li>
                    <li>read-heavy computed payloads reused across requests</li>
                </ul>

                <h2>What not to cache by default</h2>
                <ul>
                    <li>raw write-side validation decisions</li>
                    <li>request DTOs or partially validated input</li>
                    <li>high-cardinality per-request one-off results that will not be reused</li>
                    <li>transport concerns that already have cheap deterministic computation</li>
                </ul>

                <h2>Invalidation rule</h2>
                <p>
                    Invalidate from the workflow that owns the write. Do not scatter invalidation across multiple handlers, controllers, or frontend calls.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub async fn update_status(...) -> Result<Country, AppError> {
    // persist DB change first
    /* ... */

    invalidate_bootstrap_country_cache(state).await?;
    Ok(runtime_country)
}`}</code>
                </pre>

                <h2>Practical workflow pattern</h2>
                <ol>
                    <li>Define a stable key constant and TTL in the feature module.</li>
                    <li>Load through <code>remember(...)</code> around the DB-backed read path.</li>
                    <li>Invalidate from the workflow that mutates the underlying rows.</li>
                    <li>Keep the cached payload already shaped for its consumer.</li>
                </ol>

                <h2>Verification</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# cold request should populate cache
# repeated request should hit cache
# write action should invalidate and force refresh on next read
curl http://127.0.0.1:3000/api/bootstrap.js`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/bootstrap">Bootstrap Runtime</a> for runtime payload injection.</li>
                    <li><a href="#/cookbook/build-end-to-end-flow">Build an End-to-End Flow</a> for the country runtime example in a larger vertical slice.</li>
                    <li><a href="#/caching">Caching</a> for the framework-level feature overview.</li>
                </ul>
            </div>
        </div>
    )
}
