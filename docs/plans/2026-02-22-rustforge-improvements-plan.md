# Rustforge Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand Cache API, add Job failure callback, and add framework docs + 3 new cookbook chapters.

**Architecture:** Code changes are additive — new methods on existing `Cache` struct and one default-no-op method on the `Job` trait. Docs are React TSX pages following existing Prism-highlighted patterns in `core-docs/frontend/src/pages/`.

**Tech Stack:** Rust (redis, serde, async-trait), React TSX (Prism.js syntax highlighting, Tailwind prose)

---

### Task 1: Expand Cache API — TTL + Typed JSON + Convenience

**Files:**
- Modify: `core-db/src/infra/cache.rs`

**Step 1: Add TTL and convenience methods**

Add after the existing `del` method in the `impl Cache` block:

```rust
    pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()> {
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(self.key(key), value, ttl_secs).await?;
        Ok(())
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let mut conn = self.conn.lock().await;
        let val: i64 = conn.ttl(self.key(key)).await?;
        Ok(if val < 0 { None } else { Some(val) })
    }

    pub async fn forget(&self, key: &str) -> Result<()> {
        self.del(key).await
    }

    pub async fn has(&self, key: &str) -> Result<bool> {
        let mut conn = self.conn.lock().await;
        Ok(conn.exists(self.key(key)).await?)
    }
```

**Step 2: Add typed JSON methods**

Add `use serde::{de::DeserializeOwned, Serialize};` to imports, then add:

```rust
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
            None => Ok(None),
        }
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        self.set(key, &raw).await
    }

    pub async fn set_json_ex<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        self.set_ex(key, &raw, ttl_secs).await
    }
```

**Step 3: Add remember pattern**

Add `use std::future::Future;` to imports, then add:

```rust
    pub async fn remember<T, F, Fut>(&self, key: &str, ttl_secs: u64, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }
        let value = f().await?;
        self.set_json_ex(key, &value, ttl_secs).await?;
        Ok(value)
    }

    pub async fn remember_forever<T, F, Fut>(&self, key: &str, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }
        let value = f().await?;
        self.set_json(key, &value).await?;
        Ok(value)
    }
```

**Step 4: Add atomic counters**

```rust
    pub async fn increment(&self, key: &str, by: i64) -> Result<i64> {
        let mut conn = self.conn.lock().await;
        Ok(conn.incr(self.key(key), by).await?)
    }

    pub async fn decrement(&self, key: &str, by: i64) -> Result<i64> {
        let mut conn = self.conn.lock().await;
        Ok(conn.decr(self.key(key), by).await?)
    }
```

**Step 5: Add bulk operations**

```rust
    pub async fn many(&self, keys: &[&str]) -> Result<Vec<Option<String>>> {
        let mut conn = self.conn.lock().await;
        let prefixed: Vec<String> = keys.iter().map(|k| self.key(k)).collect();
        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&prefixed)
            .query_async(&mut *conn)
            .await?;
        Ok(results)
    }

    pub async fn put_many(&self, pairs: &[(&str, &str)]) -> Result<()> {
        let mut conn = self.conn.lock().await;
        let mut pipe = redis::pipe();
        for (k, v) in pairs {
            pipe.set(self.key(k), *v);
        }
        pipe.query_async::<()>(&mut *conn).await?;
        Ok(())
    }

    pub async fn flush_prefix(&self, prefix: &str) -> Result<()> {
        let mut conn = self.conn.lock().await;
        let pattern = format!("{}*", self.key(prefix));
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await?;
        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async::<()>(&mut *conn)
                .await?;
        }
        Ok(())
    }
```

**Step 6: Remove dead_code allow**

Remove `#![allow(dead_code)]` from line 1 since the API is now public and used.

**Step 7: Verify compilation**

Run: `cargo check -p core-db`
Expected: compiles with no errors.

**Step 8: Commit**

```bash
git add core-db/src/infra/cache.rs
git commit -m "feat(cache): expand Cache API with TTL, JSON, remember, counters, bulk ops"
```

---

### Task 2: Add Job `failed()` Callback

**Files:**
- Modify: `core-jobs/src/lib.rs`
- Modify: `core-jobs/src/worker.rs`

**Step 1: Add `failed()` to Job trait**

In `core-jobs/src/lib.rs`, add inside the `Job` trait after the `dispatch` method:

```rust
    /// Called when all retries are exhausted, before persisting to failed_jobs.
    /// Use for cleanup, alerts, or state reversion.
    /// Errors are logged but do not prevent failure persistence.
    async fn failed(&self, _ctx: &JobContext, _error: &str) -> anyhow::Result<()> {
        Ok(())
    }
```

**Step 2: Add `on_failed` to `JobHandler` trait**

In `core-jobs/src/worker.rs`, add a new method to `JobHandler`:

```rust
#[async_trait]
trait JobHandler: Send + Sync {
    async fn execute(
        &self,
        json: Value,
        ctx: &JobContext,
        attempts: u32,
    ) -> anyhow::Result<JobResult>;

    async fn on_failed(
        &self,
        json: Value,
        ctx: &JobContext,
        error: &str,
    ) -> anyhow::Result<()>;
}
```

**Step 3: Implement `on_failed` in `JobShim`**

Add inside the `impl<J: Job> JobHandler for JobShim<J>` block:

```rust
    async fn on_failed(
        &self,
        json: Value,
        ctx: &JobContext,
        error: &str,
    ) -> anyhow::Result<()> {
        let job: J = serde_json::from_value(json)?;
        job.failed(ctx, error).await
    }
```

**Step 4: Call `on_failed` before `persist_failure` in the standard queue path**

In `run_internal`, in the standard queue failure branch where `wrapper.attempts >= max_retries` (around line 330), add before the `self.persist_failure(...)` call:

```rust
// Call job's failed() callback
if let Some(handler) = self.registry.get(wrapper.job.as_str()) {
    if let Err(e) = handler.on_failed(wrapper.data.clone(), &self.context, &err).await {
        tracing::error!("Job failed() callback error: {}", e);
    }
}
```

**Step 5: Call `on_failed` before `persist_failure` in the grouped queue path**

Same pattern in the grouped queue failure branch (around line 276), add before `self.persist_failure(...)`:

```rust
if let Some(handler) = self.registry.get(wrapper.job.as_str()) {
    if let Err(e) = handler.on_failed(wrapper.data.clone(), &self.context, &err).await {
        tracing::error!("Job failed() callback error: {}", e);
    }
}
```

**Step 6: Verify compilation**

Run: `cargo check -p core-jobs`
Expected: compiles with no errors.

**Step 7: Commit**

```bash
git add core-jobs/src/lib.rs core-jobs/src/worker.rs
git commit -m "feat(jobs): add failed() callback to Job trait for retry exhaustion cleanup"
```

---

### Task 3: Framework Doc — Caching Page

**Files:**
- Create: `core-docs/frontend/src/pages/async/Caching.tsx`

**Step 1: Create the Caching framework doc page**

Create `core-docs/frontend/src/pages/async/Caching.tsx` following the exact TSX pattern used by other pages (import useEffect + Prism, export named function, div.space-y-10 > div.prose.prose-orange.max-w-none).

Content sections:
1. **Title**: "Caching" with subtitle "Redis-backed cache with typed values, TTL, and remember pattern."
2. **Setup**: Cache is initialized in `BootContext` from `core-db`. Available via `ctx.redis` (or add to `AppApiState`).
3. **Basic Operations**: `get`, `set`, `del`, `has`, `forget` with string examples.
4. **TTL**: `set_ex`, `ttl` — show caching a value for 5 minutes.
5. **Typed JSON**: `get_json`, `set_json`, `set_json_ex` — show caching a struct.
6. **Remember Pattern**: `remember`, `remember_forever` — show caching an expensive DB query with 10-min TTL.
7. **Atomic Counters**: `increment`, `decrement` — show view counter pattern.
8. **Bulk Operations**: `many`, `put_many`, `flush_prefix`.
9. **Cache Invalidation**: Guidance on when to invalidate (after model update/delete), using `forget` or `flush_prefix`.

All code blocks use: `<pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm"><code className="language-rust">{...}</code></pre>`

**Step 2: Commit**

```bash
git add core-docs/frontend/src/pages/async/Caching.tsx
git commit -m "docs: add Caching framework documentation page"
```

---

### Task 4: Framework Doc — Improve Responses Page

**Files:**
- Modify: `core-docs/frontend/src/pages/http/Responses.tsx`

**Step 1: Add AppError usage section and error mapping examples**

After the existing "Usage in Handlers" section, add:

1. **AppError Variants** — table listing all variants (Internal, NotFound, BadRequest, Unauthorized, Forbidden, TooManyRequests, UnprocessableEntity, Validation) with HTTP status code and error_code string.

2. **Returning Errors from Handlers** — code examples:

```rust
use core_web::error::AppError;
use core_i18n::t;

// 404
async fn get_article(Path(id): Path<i64>) -> Result<ApiResponse<ArticleView>, AppError> {
    let article = Article::find(&db, id).await?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))?;
    Ok(ApiResponse::success(article, ""))
}

// 403 with business logic
async fn delete_article(auth: Auth<WebGuard>) -> Result<ApiResponse<()>, AppError> {
    if !auth.user.can_manage_articles() {
        return Err(AppError::Forbidden(t("Not allowed to delete articles")));
    }
    // ...
}

// Validation with field errors
async fn custom_validation() -> Result<ApiResponse<()>, AppError> {
    let mut errors = HashMap::new();
    errors.insert("email".to_string(), vec!["Email already taken".to_string()]);
    Err(AppError::Validation {
        message: "Validation failed".to_string(),
        errors,
    })
}
```

3. **Auto-conversion** note — `From<E: Into<anyhow::Error>>` means any `?` on a `Result` auto-wraps to `AppError::Internal`.

**Step 2: Add `useEffect` + Prism import**

The current `Responses.tsx` is missing the Prism highlight call. Add:

```tsx
import { useEffect } from 'react'
import Prism from 'prismjs'
```

And inside the component before `return`:

```tsx
useEffect(() => {
    Prism.highlightAll()
}, [])
```

**Step 3: Commit**

```bash
git add core-docs/frontend/src/pages/http/Responses.tsx
git commit -m "docs: expand Responses page with AppError variants and error handling examples"
```

---

### Task 5: Framework Doc — Improve Attachments Page

**Files:**
- Modify: `core-docs/frontend/src/pages/framework-features/AttachmentsFeature.tsx`

**Step 1: Add end-to-end upload flow section**

After the existing "Atomicity" section, add a new section:

**"End-to-End: Multipart Upload to Attachment"**

Show the full flow:
1. Client uploads file via multipart form
2. Handler receives file, uploads to S3 storage
3. Creates `AttachmentInput` from upload result
4. Passes to model insert/update

```rust
use axum::extract::Multipart;
use core_db::infra::storage::Storage;
use core_db::platform::attachments::types::AttachmentInput;

async fn upload_article_cover(
    State(state): State<AppApiState>,
    mut multipart: Multipart,
) -> ApiResult<serde_json::Value> {
    let field = multipart.next_field().await?.ok_or(
        AppError::BadRequest("No file provided".to_string()),
    )?;

    let file_name = field.file_name().unwrap_or("upload").to_string();
    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
    let data = field.bytes().await?;

    // 1. Upload to S3
    let path = format!("uploads/articles/{}", file_name);
    state.storage.put(&path, &data, &content_type).await?;

    // 2. Create attachment input
    let input = AttachmentInput::new(
        &path,
        &content_type,
        data.len() as u64,
        None, // width (optional, set if image)
        None, // height (optional, set if image)
    );

    // 3. Attach to model
    let article = Article::new(&state.db)
        .update()
        .where_id(Op::Eq, article_id)
        .set_attachment_cover(input)
        .save()
        .await?;

    Ok(ApiResponse::success(
        serde_json::json!({ "cover_url": article.cover_url }),
        &t("Cover uploaded"),
    ))
}
```

**Step 2: Commit**

```bash
git add core-docs/frontend/src/pages/framework-features/AttachmentsFeature.tsx
git commit -m "docs: add end-to-end upload flow to Attachments page"
```

---

### Task 6: Cookbook Chapter 10 — Caching Recipe

**Files:**
- Create: `core-docs/frontend/src/pages/cookbook/Chapter10CachingRecipe.tsx`

**Step 1: Create the cookbook chapter**

Follow the exact cookbook TSX pattern from Chapter 3. Structure:

```
Chapter 10: Caching Recipe
├── Step 0: Scope and Defaults
│   - Cache (core_db::infra::cache::Cache) is in BootContext as ctx.redis
│   - Redis must be running and configured via REDIS_URL
├── Step 1: Add Cache to AppApiState
│   - File: app/src/internal/api/state.rs
│   - Add `pub cache: Cache` field
│   - Initialize from ctx.redis.clone()
├── Step 2: Cache Expensive Query with remember()
│   - File: app/src/internal/api/v1/article.rs
│   - Show handler using state.cache.remember() to cache article list for 5min
│   - Cache key pattern: "articles:list:{page}"
├── Step 3: Invalidate on Write
│   - File: app/src/internal/workflows/article/create_article.rs
│   - After insert, call state.cache.flush_prefix("articles:").await
│   - Same in update and delete workflows
├── Step 4: Counter Pattern
│   - File: app/src/internal/api/v1/article.rs
│   - Show view count with state.cache.increment("article:views:{id}", 1)
│   - Periodic flush to DB via scheduled job
├── Step 5: Verify
│   - curl commands to hit cached endpoint twice
│   - redis-cli GET to inspect cached value
│   - redis-cli TTL to verify expiration
├── Decision Rule
│   - Cache when: same query > 10 req/min, data changes < 1/min
│   - Don't cache when: user-specific data, real-time accuracy needed
│   - Always invalidate on write, never rely on TTL alone for consistency
```

**Step 2: Commit**

```bash
git add core-docs/frontend/src/pages/cookbook/Chapter10CachingRecipe.tsx
git commit -m "docs: add Cookbook Chapter 10 - Caching Recipe"
```

---

### Task 7: Cookbook Chapter 11 — Testing Recipe

**Files:**
- Create: `core-docs/frontend/src/pages/cookbook/Chapter11TestingRecipe.tsx`

**Step 1: Create the cookbook chapter**

Structure:

```
Chapter 11: Testing Recipe
├── Step 0: Scope
│   - Uses cargo test with #[tokio::test]
│   - Test database: separate DB or transaction rollback
│   - Framework provides no test harness — you build on standard Rust testing
├── Step 1: Test Helper Setup
│   - File: app/tests/helpers/mod.rs
│   - Create test_db() → PgPool (connects to TEST_DATABASE_URL)
│   - Create test_cache() → Cache (connects to test Redis)
│   - Create test_state() → AppApiState with test deps
├── Step 2: Test Contracts (Validation)
│   - File: app/tests/contracts/article_test.rs
│   - Test valid input passes validation
│   - Test invalid input returns field errors
│   - Use validator::Validate trait directly
│   - Example:
│     let input = CreateArticleInput { title: "".into(), ... };
│     let result = input.validate();
│     assert!(result.is_err());
│     let errors = result.unwrap_err();
│     assert!(errors.field_errors().contains_key("title"));
├── Step 3: Test Handlers (HTTP layer)
│   - File: app/tests/api/article_test.rs
│   - Build router: let app = Router::new().merge(article::router()).with_state(test_state);
│   - Use axum::body::Body + tower::ServiceExt::oneshot
│   - Send request, assert status code + response body
│   - Example:
│     let req = Request::builder().uri("/articles").body(Body::empty())?;
│     let res = app.oneshot(req).await?;
│     assert_eq!(res.status(), StatusCode::OK);
├── Step 4: Test Jobs
│   - File: app/tests/jobs/rebuild_index_test.rs
│   - Instantiate job directly, call handle() with test JobContext
│   - Assert side effects (DB changes, etc.)
│   - Example:
│     let job = RebuildArticleIndexJob { article_id: 1 };
│     let ctx = test_job_context().await;
│     job.handle(&ctx).await.unwrap();
│     // assert DB state changed
├── Step 5: Integration Test
│   - File: app/tests/integration/article_crud_test.rs
│   - Full cycle: create → read → update → delete
│   - Use test_state() with real test DB
│   - Wrap in transaction and rollback after test
├── Step 6: Run Tests
│   - cargo test -p app
│   - cargo test -p app -- --test-threads=1 (if tests share DB state)
│   - TEST_DATABASE_URL=postgres://... cargo test
├── Decision Rule
│   - Unit test: contracts, pure logic, job handlers
│   - Integration test: full request → DB → response cycle
│   - Don't test: generated code, framework internals
│   - Always test: custom validation rules, business logic in workflows
```

**Step 2: Commit**

```bash
git add core-docs/frontend/src/pages/cookbook/Chapter11TestingRecipe.tsx
git commit -m "docs: add Cookbook Chapter 11 - Testing Recipe"
```

---

### Task 8: Cookbook Chapter 12 — Event Fan-Out Recipe

**Files:**
- Create: `core-docs/frontend/src/pages/cookbook/Chapter12EventFanOutRecipe.tsx`

**Step 1: Create the cookbook chapter**

Structure:

```
Chapter 12: Event Fan-Out Recipe
├── Step 0: Scope
│   - In Rustforge, Jobs + Notify IS the event system
│   - "Event" = dispatching a fan-out job
│   - "Listeners" = the job handler logic that fans out
│   - No separate Event abstraction needed
├── Step 1: Define Fan-Out Job
│   - File: app/src/internal/jobs/definitions/order_placed_fanout.rs
│   - OrderPlacedFanoutJob { order_id: i64, customer_email: String }
│   - impl Job with NAME = "order.placed.fanout"
├── Step 2: Job Handler Fans Out
│   - In handle():
│     1. Send confirmation email via MailChannel
│     2. Push realtime event via RealtimePublisher
│     3. Call external webhook (HTTP POST)
│   - Each channel is independent — partial failure shouldn't block others
│   - Example with try-each pattern:
│     if let Err(e) = send_email(...).await { tracing::error!("email: {e}"); }
│     if let Err(e) = push_realtime(...).await { tracing::error!("realtime: {e}"); }
│     if let Err(e) = call_webhook(...).await { tracing::error!("webhook: {e}"); }
│     Ok(())
├── Step 3: Transactional Dispatch
│   - In order creation workflow, use JobBuffer
│   - Job only dispatched if order INSERT commits
│   - Same pattern as Chapter 3 Step 5
├── Step 4: Idempotency Per Channel
│   - Email: use idempotency key (order_id) to prevent duplicate sends
│   - Realtime: client-side dedup via event ID
│   - Webhook: include X-Idempotency-Key header
│   - If job retries, channels must handle duplicate calls
├── Step 5: Add failed() for Alerting
│   - Implement failed() on the fan-out job
│   - Send Slack/PagerDuty alert when all retries exhausted
│   - Example:
│     async fn failed(&self, ctx: &JobContext, error: &str) -> anyhow::Result<()> {
│         tracing::error!("ALERT: Order {} fanout failed: {}", self.order_id, error);
│         // notify ops channel
│         Ok(())
│     }
├── Step 6: Verify
│   - curl to create order
│   - Check: email sent, realtime event pushed, webhook called
│   - Check: failed_jobs table empty
├── Decision Rule
│   - Direct dispatch: single side effect (just send email)
│   - Fan-out job: 2+ independent side effects from one event
│   - Separate jobs: side effects have different retry/backoff needs
```

**Step 2: Commit**

```bash
git add core-docs/frontend/src/pages/cookbook/Chapter12EventFanOutRecipe.tsx
git commit -m "docs: add Cookbook Chapter 12 - Event Fan-Out Recipe"
```

---

### Task 9: Wire Up Routes — App.tsx, Sidebar.tsx, CookbookOverview.tsx

**Files:**
- Modify: `core-docs/frontend/src/App.tsx`
- Modify: `core-docs/frontend/src/components/Sidebar.tsx`
- Modify: `core-docs/frontend/src/pages/cookbook/CookbookOverview.tsx`

**Step 1: Update App.tsx**

Add imports after line 24 (after Chapter9ProductionHardening import):

```tsx
import { Chapter10CachingRecipe } from './pages/cookbook/Chapter10CachingRecipe'
import { Chapter11TestingRecipe } from './pages/cookbook/Chapter11TestingRecipe'
import { Chapter12EventFanOutRecipe } from './pages/cookbook/Chapter12EventFanOutRecipe'
```

Add import for Caching page after Scheduler import (line 66):

```tsx
import { Caching } from './pages/async/Caching'
```

Add to routeMap after `'#/cookbook-chapter-9-production-hardening'` entry:

```tsx
    '#/cookbook-chapter-10-caching': Chapter10CachingRecipe,
    '#/cookbook-chapter-11-testing': Chapter11TestingRecipe,
    '#/cookbook-chapter-12-event-fanout': Chapter12EventFanOutRecipe,
```

Add to routeMap after `'#/scheduler'` entry:

```tsx
    '#/caching': Caching,
```

**Step 2: Update Sidebar.tsx**

In the Cookbook section items array, add after Chapter 9 entry:

```tsx
            { title: 'Chapter 10: Caching', href: '#/cookbook-chapter-10-caching' },
            { title: 'Chapter 11: Testing', href: '#/cookbook-chapter-11-testing' },
            {
                title: 'Chapter 12: Event Fan-Out',
                href: '#/cookbook-chapter-12-event-fanout',
            },
```

In the "Async & Jobs" section items array, add after Cron Scheduler entry:

```tsx
            { title: 'Caching', href: '#/caching' },
```

**Step 3: Update CookbookOverview.tsx**

Add to the "Available Chapters" grid (after Chapter 9 link):

```tsx
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
```

Update the reading order section — add after the existing ordered list item 7:

```tsx
                    <li>Chapters 10-12 for caching, testing, and event fan-out patterns.</li>
```

**Step 4: Verify the docs build**

Run: `cd core-docs/frontend && npm run build` (or `npx vite build` / `npx tsc --noEmit`)
Expected: no TypeScript errors, build completes.

**Step 5: Commit**

```bash
git add core-docs/frontend/src/App.tsx core-docs/frontend/src/components/Sidebar.tsx core-docs/frontend/src/pages/cookbook/CookbookOverview.tsx
git commit -m "docs: wire up new cookbook chapters and caching page in router and sidebar"
```

---

## Task Dependency Summary

```
Task 1 (Cache code) ─────► Task 3 (Cache docs) ─────► Task 6 (Cookbook Ch10)
Task 2 (Job failed) ─────────────────────────────────► Task 8 (Cookbook Ch12)
                           Task 4 (Responses docs)     Task 7 (Cookbook Ch11)
                           Task 5 (Attachments docs)
                                                        All ──► Task 9 (Wiring)
```

Tasks 1 & 2 are independent (parallel).
Tasks 3, 4, 5 are independent (parallel, after Task 1).
Tasks 6, 7, 8 are independent (parallel, after Tasks 1 & 2).
Task 9 depends on Tasks 3, 6, 7, 8 (all new pages must exist first).
