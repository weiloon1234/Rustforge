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
                    Chapter 10: Caching Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Add Redis caching to API endpoints with remember pattern, invalidation on writes,
                    and atomic counters.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Scope and Defaults</h2>
                <ul>
                    <li>
                        Cache (<code>core_db::infra::cache::Cache</code>) is available in BootContext
                        as <code>ctx.redis</code>.
                    </li>
                    <li>
                        Redis must be running and configured via <code>REDIS_URL</code>.
                    </li>
                    <li>
                        This chapter assumes the Chapter 1 API base already exists.
                    </li>
                </ul>

                <h2>Step 1: Add Cache to AppApiState</h2>
                <h3>
                    File: <code>app/src/internal/api/state.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`use core_db::infra::cache::Cache;

#[derive(Clone)]
pub struct AppApiState {
    pub db: PgPool,
    pub queue: RedisQueue,
    pub cache: Cache,
    pub cdn_base: Option<String>,
}

impl AppApiState {
    pub fn new(ctx: &bootstrap::boot::BootContext) -> Self {
        Self {
            db: ctx.db.clone(),
            queue: ctx.queue.clone(),
            cache: ctx.redis.clone(),
            cdn_base: ctx.settings.cdn.base_url.clone(),
        }
    }
}`}</code>
                </pre>

                <h2>Step 2: Cache Expensive Query with remember()</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/article.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`async fn list_articles(
    State(state): State<AppApiState>,
    Query(params): Query<ListParams>,
) -> ApiResult<Vec<ArticleView>> {
    let cache_key = format!("articles:list:page:{}", params.page.unwrap_or(1));

    let articles = state.cache.remember(&cache_key, 300, || async {
        Article::new(&state.db)
            .query()
            .order_by(ArticleCol::CreatedAt, Desc)
            .paginate(params.page.unwrap_or(1), 20)
            .all()
            .await
    }).await?;

    Ok(ApiResponse::success(articles, ""))
}`}</code>
                </pre>

                <h2>Step 3: Invalidate on Write</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/article.rs</code> (or workflow)
                </h3>
                <p>
                    <strong>Create:</strong>
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`async fn create_article(
    State(state): State<AppApiState>,
    ContractJson(input): ContractJson<CreateArticleInput>,
) -> ApiResult<ArticleView> {
    let article = Article::new(&state.db)
        .insert()
        .set_title(&input.title)
        .save()
        .await?;

    // Invalidate cached lists
    state.cache.flush_prefix("articles:").await?;

    Ok(ApiResponse::created(article, &t("Article created")))
}`}</code>
                </pre>
                <p>
                    <strong>Update:</strong>
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`async fn update_article(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
    ContractJson(input): ContractJson<UpdateArticleInput>,
) -> ApiResult<ArticleView> {
    let article = Article::find(&state.db, id).await?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))?;

    let updated = article
        .update()
        .set_title(&input.title)
        .save()
        .await?;

    // Invalidate cached lists and the individual article cache
    state.cache.flush_prefix("articles:").await?;
    state.cache.forget(&format!("article:{}", id)).await?;

    Ok(ApiResponse::success(updated, &t("Article updated")))
}`}</code>
                </pre>
                <p>
                    <strong>Delete:</strong>
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`async fn delete_article(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
) -> ApiResult<()> {
    let article = Article::find(&state.db, id).await?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))?;

    article.delete().await?;

    // Invalidate cached lists and the individual article cache
    state.cache.flush_prefix("articles:").await?;
    state.cache.forget(&format!("article:{}", id)).await?;

    Ok(ApiResponse::success((), &t("Article deleted")))
}`}</code>
                </pre>

                <h2>Step 4: Counter Pattern</h2>
                <h3>
                    File: <code>app/src/internal/api/v1/article.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`async fn view_article(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
) -> ApiResult<ArticleView> {
    let article = Article::find(&state.db, id).await?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))?;

    // Increment view counter (fire-and-forget)
    let _ = state.cache.increment(&format!("article:views:{}", id), 1).await;

    Ok(ApiResponse::success(article, ""))
}`}</code>
                </pre>
                <p>
                    <strong>Note:</strong> Atomic counters live in Redis only. Use a scheduled job
                    to periodically flush accumulated counts back to the database so they survive
                    Redis restarts and appear in queries/reports.
                </p>

                <h2>Step 5: Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`# First request (cache miss — hits DB)
curl http://127.0.0.1:3000/api/v1/user/articles?page=1

# Second request (cache hit — no DB query)
curl http://127.0.0.1:3000/api/v1/user/articles?page=1

# Inspect cache
redis-cli GET "myapp:dev:articles:list:page:1"

# Check TTL
redis-cli TTL "myapp:dev:articles:list:page:1"

# Create article (triggers invalidation)
curl -X POST http://127.0.0.1:3000/api/v1/user/articles \\
  -H 'Content-Type: application/json' \\
  -d '{"title":"New Article"}'

# Verify cache cleared
redis-cli GET "myapp:dev:articles:list:page:1"
# (nil)`}</code>
                </pre>

                <h2>Chapter Decision Rule</h2>
                <ul>
                    <li>
                        Cache when: same query &gt; 10 req/min and data changes &lt; 1/min.
                    </li>
                    <li>
                        Don't cache when: user-specific data or real-time accuracy needed.
                    </li>
                    <li>
                        Always invalidate on write — never rely on TTL alone for consistency.
                    </li>
                    <li>
                        Use <code>flush_prefix()</code> for groups of related keys.
                    </li>
                    <li>
                        Use <code>forget()</code> for individual known keys.
                    </li>
                </ul>
            </div>
        </div>
    )
}
