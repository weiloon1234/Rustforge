import { useEffect } from 'react'
import Prism from 'prismjs'

export function Caching() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Caching</h1>
                <p className="text-xl text-gray-500">
                    Redis-backed cache with typed values, TTL, and remember pattern.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h3>Setup</h3>
                <p>
                    The cache is initialized during bootstrap via{' '}
                    <code>core_db::infra::cache::Cache</code>. It connects to Redis using the
                    settings in your <code>.env</code> and is available on the{' '}
                    <code>BootContext</code> as <code>ctx.redis</code>. To use it in handlers, add
                    it to your application state.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// bootstrap/src/boot.rs
let redis = core_db::infra::cache::create_cache(&settings.redis).await?;

// The BootContext exposes it as ctx.redis
pub struct BootContext {
    pub db: PgPool,
    pub redis: core_db::infra::cache::Cache,
    // ...
}`}</code>
                </pre>
                <p>
                    In your app state (or <code>FrameworkState</code>), the cache is available as a
                    cloneable handle. Axum&apos;s <code>FromRef</code> lets handlers extract it
                    directly.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use axum::extract::State;
use core_db::infra::cache::Cache;

async fn show_article(
    State(cache): State<Cache>,
    State(db): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<Article>> {
    // cache is ready to use
    let key = format!("article:{}", slug);
    // ...
}`}</code>
                </pre>

                <h3>Basic Operations</h3>
                <p>
                    The core operations are <code>get</code>, <code>set</code>, <code>del</code>,{' '}
                    <code>has</code>, and <code>forget</code>. All keys are automatically prefixed
                    with the configured Redis prefix.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Store a value
cache.set("site:announcement", "Maintenance at 2 AM UTC").await?;

// Retrieve it
let announcement = cache.get("site:announcement").await?;
// => Some("Maintenance at 2 AM UTC")

// Check existence
let exists = cache.has("site:announcement").await?;
// => true

// Delete a key (del and forget are equivalent)
cache.del("site:announcement").await?;
cache.forget("site:announcement").await?;

// After deletion
let gone = cache.get("site:announcement").await?;
// => None`}</code>
                </pre>

                <h3>TTL (Time-To-Live)</h3>
                <p>
                    Use <code>set_ex</code> to cache a value with an expiration in seconds, and{' '}
                    <code>ttl</code> to check how much time remains.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Cache a session token for 5 minutes (300 seconds)
cache.set_ex("session:abc123", &token, 300).await?;

// Check remaining TTL
let remaining = cache.ttl("session:abc123").await?;
// => Some(297)  — seconds left

// After expiry, ttl returns None
// => None`}</code>
                </pre>

                <h3>Typed JSON</h3>
                <p>
                    For structured data, use <code>get_json</code>, <code>set_json</code>, and{' '}
                    <code>set_json_ex</code>. The cache serializes to JSON internally so any{' '}
                    <code>Serialize + DeserializeOwned</code> type works.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

// Cache a user profile for 10 minutes
let profile = UserProfile {
    id: user.id,
    name: user.name.clone(),
    email: user.email.clone(),
    avatar_url: user.avatar_url.clone(),
};
cache.set_json_ex("user:profile:abc", &profile, 600).await?;

// Retrieve the typed value — no manual deserialization needed
let cached: Option<UserProfile> = cache.get_json("user:profile:abc").await?;
if let Some(p) = cached {
    tracing::info!("Cached user: {} <{}>", p.name, p.email);
}

// Store without TTL (lives until explicitly deleted)
cache.set_json("app:feature_flags", &flags).await?;`}</code>
                </pre>

                <h3>Remember Pattern</h3>
                <p>
                    The <code>remember</code> method checks the cache first. On a miss it calls your
                    closure, stores the result, and returns it. This is the recommended way to cache
                    expensive queries.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Cache a DB query for 10 minutes (600 seconds).
// On cache hit, the closure is never called.
let articles = cache
    .remember("articles:trending", 600, || async {
        sqlx::query_as::<_, Article>(
            "SELECT * FROM articles ORDER BY view_count DESC LIMIT 20"
        )
        .fetch_all(&db)
        .await
        .map_err(Into::into)
    })
    .await?;

// remember_forever — same pattern, but no expiration.
// Useful for data that rarely changes (e.g. country list, config).
let countries = cache
    .remember_forever("geo:countries", || async {
        sqlx::query_as::<_, Country>("SELECT * FROM countries ORDER BY name")
            .fetch_all(&db)
            .await
            .map_err(Into::into)
    })
    .await?;`}</code>
                </pre>

                <h3>Atomic Counters</h3>
                <p>
                    Use <code>increment</code> and <code>decrement</code> for lock-free atomic
                    counters. These map directly to Redis INCR/DECR and return the new value.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Track page views for an article
let view_key = format!("article:{}:views", article.id);
let new_count = cache.increment(&view_key, 1).await?;
tracing::info!("Article {} now has {} views", article.id, new_count);

// Decrement remaining invites
let invite_key = format!("user:{}:invites_remaining", user.id);
let remaining = cache.decrement(&invite_key, 1).await?;
if remaining < 0 {
    // User exceeded invite limit — reset and reject
    cache.increment(&invite_key, 1).await?;
    return Err(AppError::forbidden("No invites remaining"));
}`}</code>
                </pre>

                <h3>Bulk Operations</h3>
                <p>
                    Fetch or store multiple keys in a single round-trip with <code>many</code> and{' '}
                    <code>put_many</code>. Use <code>flush_prefix</code> to delete all keys matching
                    a prefix.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// Fetch multiple keys at once (MGET)
let values = cache
    .many(&["user:1:name", "user:2:name", "user:3:name"])
    .await?;
// => [Some("Alice"), Some("Bob"), None]

// Store multiple keys at once (pipelined SET)
cache
    .put_many(&[
        ("config:maintenance", "false"),
        ("config:motd", "Welcome back!"),
        ("config:version", "1.4.2"),
    ])
    .await?;

// Delete all keys under a prefix (e.g. after a bulk import)
cache.flush_prefix("article:").await?;
// Deletes article:1, article:2, article:trending, etc.`}</code>
                </pre>

                <h3>Cache Invalidation</h3>
                <p>
                    Stale cache is one of the hardest problems in software. Follow these guidelines
                    to keep your data consistent:
                </p>
                <ul>
                    <li>
                        <strong>Invalidate after model writes.</strong> Whenever you insert, update,
                        or delete a record, immediately forget the related cache keys. Do this in
                        the same service method, not as an afterthought.
                    </li>
                    <li>
                        <strong>
                            Use <code>forget()</code> for specific keys
                        </strong>{' '}
                        and{' '}
                        <strong>
                            <code>flush_prefix()</code> for groups.
                        </strong>{' '}
                        Prefer targeted invalidation over flushing everything.
                    </li>
                    <li>
                        <strong>Don&apos;t rely solely on TTL for consistency.</strong> TTL is a
                        safety net, not a primary invalidation strategy. A 10-minute TTL means users
                        could see stale data for up to 10 minutes after a write.
                    </li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`// After updating an article, invalidate all related cache entries
pub async fn update_article(
    cache: &Cache,
    db: &PgPool,
    id: Uuid,
    payload: UpdateArticle,
) -> Result<Article> {
    let article = sqlx::query_as::<_, Article>(
        "UPDATE articles SET title = $1, body = $2 WHERE id = $3 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.body)
    .bind(id)
    .fetch_one(db)
    .await?;

    // Invalidate specific key
    cache.forget(&format!("article:{}", article.slug)).await?;

    // Invalidate listing caches that include this article
    cache.flush_prefix("articles:").await?;

    Ok(article)
}`}</code>
                </pre>
            </div>
        </div>
    )
}
