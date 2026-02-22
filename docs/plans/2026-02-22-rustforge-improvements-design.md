# Rustforge Improvements Design

## Scope

Two code additions + documentation overhaul across framework docs and cookbook.

---

## Part A: Code — Cache Abstraction

**File**: `core-db/src/infra/cache.rs`

Expand the existing `Cache` struct (currently only `get`/`set`/`del` with strings) with:

### New Methods

```rust
// TTL support
pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()>
pub async fn ttl(&self, key: &str) -> Result<Option<i64>>

// Typed JSON serialization
pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>
pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()>
pub async fn set_json_ex<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> Result<()>

// Remember pattern (get-or-compute + cache)
pub async fn remember<T, F, Fut>(&self, key: &str, ttl_secs: u64, f: F) -> Result<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>

pub async fn remember_forever<T, F, Fut>(&self, key: &str, f: F) -> Result<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>

// Convenience
pub async fn forget(&self, key: &str) -> Result<()>  // alias for del
pub async fn has(&self, key: &str) -> Result<bool>

// Atomic counters
pub async fn increment(&self, key: &str, by: i64) -> Result<i64>
pub async fn decrement(&self, key: &str, by: i64) -> Result<i64>

// Bulk operations
pub async fn many(&self, keys: &[&str]) -> Result<Vec<Option<String>>>
pub async fn put_many(&self, pairs: &[(&str, &str)]) -> Result<()>
pub async fn flush_prefix(&self, prefix: &str) -> Result<()>
```

### Design Decisions

- **No `Cache::tags()`** — requires SET tracking per tag, adds complexity for marginal benefit. `flush_prefix()` covers 80% use case.
- **JSON methods use serde** — `serde_json::to_string` / `from_str` for typed values.
- **`remember` is async-closure** — closure only called on cache miss.
- **Backward compatible** — all new methods, no changes to existing API.

---

## Part B: Code — Job `failed()` Callback

**File**: `core-jobs/src/lib.rs`

Add one method to the `Job` trait:

```rust
#[async_trait]
pub trait Job: Serialize + DeserializeOwned + Send + Sync + Debug + 'static {
    // ... existing methods unchanged ...

    /// Called when all retries are exhausted, before persisting to failed_jobs table.
    /// Use for cleanup, alerts, or state reversion.
    async fn failed(&self, _ctx: &JobContext, _error: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
```

**File**: `core-jobs/src/worker.rs`

Modify `JobShim` to also carry the deserialized job for calling `failed()`, and update `persist_failure` to invoke it. Specifically:

1. Change `JobResult::Failure` to include the serialized job data.
2. In the worker's failure path (both standard and grouped), when `attempts >= max_retries`, call the job's `failed()` method before `persist_failure()`.

### Design Decisions

- **Default no-op** — existing jobs don't break.
- **Called before DB persist** — so the callback can reference the job context.
- **Error in `failed()` is logged but doesn't prevent persist** — the failure record still goes to DB.

---

## Part C: Framework Documentation Improvements

### C1. New Page — Caching (`core-docs/frontend/src/pages/async/Caching.tsx`)

Framework-level "how to use" doc for the Cache API:
- Cache setup (already in BootContext)
- Basic get/set/del with strings
- Typed JSON caching
- TTL and expiration
- Remember pattern for expensive queries
- Atomic counters
- Bulk operations
- Cache invalidation guidance

### C2. Improve Existing Pages

- **DataTable** (`AutoDataTable.tsx`): Add concrete registration and mounting example showing how to wire a datatable definition to a route.
- **Attachments** (`AttachmentsFeature.tsx`): Add end-to-end upload flow (multipart → S3 → model association).
- **Responses** (`Responses.tsx`): Add custom error variant examples and error mapping patterns.

---

## Part D: Cookbook Additions

### Chapter 10: Caching Recipe (`Chapter10CachingRecipe.tsx`)

Step-by-step team-standard recipe:
- Step 0: Scope (Cache is in BootContext, available via AppApiState)
- Step 1: Add cache to AppApiState
- Step 2: Cache expensive query results with `remember()`
- Step 3: Invalidate on model update/delete
- Step 4: Counter pattern (rate limiting, view counts)
- Step 5: Verify with curl + Redis CLI
- Decision Rule: when to cache vs. when to query directly

### Chapter 11: Testing Recipe (`Chapter11TestingRecipe.tsx`)

Step-by-step team-standard for testing:
- Step 0: Scope (test infrastructure, test database, fixtures)
- Step 1: Create test helper module (`app/tests/helpers/mod.rs`)
- Step 2: Test contracts (validation pass/fail assertions)
- Step 3: Test handlers (mock state, axum test utilities)
- Step 4: Test jobs (dispatch + verify side effects)
- Step 5: Integration test (full HTTP request → response cycle)
- Step 6: Run tests with `cargo test`
- Decision Rule: unit vs. integration test boundaries

### Chapter 12: Event Fan-Out Recipe (`Chapter12EventFanOutRecipe.tsx`)

Clarify that Jobs + Notify IS the event system:
- Step 0: Scope (event = job dispatch, listeners = job handler logic)
- Step 1: Define a fan-out job (e.g., `OrderPlacedFanoutJob`)
- Step 2: Job handler dispatches to multiple channels (mail, realtime, external webhook)
- Step 3: Transactional dispatch with `JobBuffer`
- Step 4: Handling partial failures in fan-out (idempotency per channel)
- Decision Rule: direct dispatch vs. fan-out job vs. separate jobs

### Cookbook Overview Update

Add Chapters 10-12 to the `CookbookOverview.tsx` available chapters list and update the reading order.

### App.tsx + Sidebar.tsx Updates

Register new routes and sidebar entries for:
- `#/caching` framework doc page
- `#/cookbook-chapter-10-caching` cookbook chapter
- `#/cookbook-chapter-11-testing` cookbook chapter
- `#/cookbook-chapter-12-event-fanout` cookbook chapter

---

## Implementation Order

1. Cache abstraction (code) — foundational for docs
2. Job `failed()` callback (code) — small change
3. Framework doc: Caching page
4. Framework doc: fix existing pages (DataTable, Attachments, Responses)
5. Cookbook Chapter 10: Caching Recipe
6. Cookbook Chapter 11: Testing Recipe
7. Cookbook Chapter 12: Event Fan-Out Recipe
8. Wire up routes (App.tsx, Sidebar.tsx, CookbookOverview.tsx)

---

## Out of Scope

- `Cache::tags()` — YAGNI
- ActiveRecord expansion — user chose not to prioritize
- Event/Listener abstraction — existing Jobs + Notify pattern is sufficient
- Deployment/Docker guides
- Multi-tenancy patterns
