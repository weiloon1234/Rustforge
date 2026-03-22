# Observer Mutation Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable model observer before-hooks (`on_creating`, `on_updating`, `on_deleting`) to inject field values, recalculate columns, or prevent operations — matching Laravel's model event capabilities.

**Architecture:** Add `ObserverAction` enum to `core-db`. Change before-hook return types from `Result<()>` to `Result<ObserverAction>`. Update code generator (`db-gen`) to handle `ObserverAction::Modify` (apply overrides to state) and `ObserverAction::Prevent` (abort). Migrate Rustforge scaffold and mediaforge observers to the new signatures, eliminating mediaforge's `CompositeModelObserver` in favor of a single `AppModelObserver`.

**Tech Stack:** Rust, sqlx, serde_json, async_trait, code generation via `db-gen`

**Spec:** `docs/superpowers/specs/2026-03-22-observer-mutation-design.md`

---

## File Structure

### Rustforge (core framework)

| File | Action | Responsibility |
|------|--------|----------------|
| `core-db/src/common/model_observer.rs` | Modify | Add `ObserverAction` enum, change trait signatures |
| `db-gen/src/gen_models.rs` | Modify | Generate `ObserverAction` handling in create/update/delete, generate per-model `apply_create_overrides`, `apply_update_overrides`, `convert_delete_to_update` helpers |
| `db-gen/tests/fixtures/full_stack/expected/models/article.rs` | Modify | Update snapshot to match new generated code |
| `scaffold/template/app/src/internal/observers/model.rs` | Modify | Update `AppModelObserver` before-hook return types + dispatch macros |
| `scaffold/template/app/src/internal/observers/models/*.rs` (12 files) | Modify | Update before-hook stub return types |

### Mediaforge (app)

| File | Action | Responsibility |
|------|--------|----------------|
| `mediaforge/app/src/internal/observers/model.rs` | Rewrite | Replace 3 structs with single `AppModelObserver`, update dispatch macros, update helper functions |
| `mediaforge/app/src/internal/observers/models/*.rs` (12 files) | Modify | Update before-hook stub return types |
| `mediaforge/app/src/internal/workflows/user_auth.rs` | Modify | Update `with_domain_model_observer` → `with_model_observer` calls |
| `mediaforge/app/src/internal/middleware/auth.rs` | Modify | Update `with_domain_and_audit_model_observer` → `with_model_observer` call |

---

## Task 1: Add `ObserverAction` enum and update trait signatures

**Files:**
- Modify: `core-db/src/common/model_observer.rs`

- [ ] **Step 1: Add `ObserverAction` enum**

Add above the `ModelObserver` trait definition in `core-db/src/common/model_observer.rs`:

```rust
/// Result of a "before" observer hook.
pub enum ObserverAction {
    /// Continue with no modifications.
    Continue,
    /// Continue but apply these field overrides before executing.
    /// Keys must be database column names. Value must be a JSON object.
    Modify(serde_json::Value),
    /// Abort the operation with this error.
    Prevent(anyhow::Error),
}
```

- [ ] **Step 2: Change before-hook return types**

Update `on_creating`, `on_updating`, `on_deleting` in the `ModelObserver` trait:

```rust
async fn on_creating(
    &self,
    _event: &ModelEvent,
    _new_data: &serde_json::Value,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

async fn on_updating(
    &self,
    _event: &ModelEvent,
    _old_data: &serde_json::Value,
    _changes: &serde_json::Value,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

async fn on_deleting(
    &self,
    _event: &ModelEvent,
    _old_data: &serde_json::Value,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
```

After-hooks (`on_created`, `on_updated`, `on_deleted`) stay unchanged — `Result<()>`.

- [ ] **Step 3: Update the `pub use` / module exports**

Ensure `ObserverAction` is exported alongside `ModelEvent`, `ModelObserver`, `try_get_observer`, `scope_observer`.

- [ ] **Step 4: Verify core-db compiles**

Run: `cargo check -p core-db`
Expected: PASS (no downstream consumers yet)

- [ ] **Step 5: Commit**

```bash
git add core-db/src/common/model_observer.rs
git commit -m "feat(core-db): add ObserverAction enum and update before-hook return types"
```

---

## Task 2: Update code generator — create path

**Files:**
- Modify: `db-gen/src/gen_models.rs` (around lines 3144-3198)

- [ ] **Step 1: Add `ObserverAction` to generated imports**

Find where the generated code writes the `use core_db::common::model_observer::{ModelEvent, try_get_observer};` import line. Add `ObserverAction` to the import list. Search for `model_observer::` in `gen_models.rs` to find the exact location.

- [ ] **Step 2: Change generated `save(self)` to `save(mut self)`**

In the create path (around line 3144-3146), change the generated signature:

```rust
// Before:
"    pub async fn save(self) -> Result<{record_ident}> {{"
// After:
"    pub async fn save(mut self) -> Result<{record_ident}> {{"
```

- [ ] **Step 3: Update observer call to handle `ObserverAction`**

Replace the current generated `on_creating` call block (lines ~3159-3181). The new generated code should:

1. Call `observer.on_creating(&event, &data).await?` and capture the result as `action`
2. Match on `action`:
   - `ObserverAction::Prevent(err)` → `return Err(err)`
   - `ObserverAction::Modify(overrides)` → `self.state = Self::apply_create_overrides(self.state, overrides)?`
   - `ObserverAction::Continue` → no-op

The generated code pattern:

```rust
writeln!(out, "                let action = observer.on_creating(&event, &data).await?;").unwrap();
writeln!(out, "                match action {{").unwrap();
writeln!(out, "                    ObserverAction::Prevent(err) => return Err(err),").unwrap();
writeln!(out, "                    ObserverAction::Modify(overrides) => {{").unwrap();
writeln!(out, "                        self.state = Self::apply_create_overrides(self.state, overrides)?;").unwrap();
writeln!(out, "                    }}").unwrap();
writeln!(out, "                    ObserverAction::Continue => {{}}").unwrap();
writeln!(out, "                }}").unwrap();
```

- [ ] **Step 4: Generate `apply_create_overrides` helper**

After the `to_create_input` function, generate a new function `apply_create_overrides` for each observed model. This function:

1. Takes `state: CreateState` and `overrides: serde_json::Value`
2. Validates `overrides` is a JSON object, else returns error
3. Iterates the object keys
4. For each key, matches against known column names and converts the JSON value to the appropriate `BindValue`
5. Calls `state.set_col(col_name, bind_value)` for each override
6. Returns error for unknown column names

Use the same column→type mapping already used in `to_create_input`. The generated code should look like:

```rust
fn apply_create_overrides(mut state: CreateState<'_>, overrides: serde_json::Value) -> Result<CreateState<'_>> {
    let map = overrides.as_object()
        .ok_or_else(|| anyhow::anyhow!("observer overrides must be a JSON object"))?;
    for (key, val) in map {
        match key.as_str() {
            "id" => {
                let v: i64 = serde_json::from_value(val.clone())?;
                state = state.set_col("id", v.into());
            }
            "author_id" => {
                let v: i64 = serde_json::from_value(val.clone())?;
                state = state.set_col("author_id", v.into());
            }
            // ... one arm per column ...
            other => anyhow::bail!("unknown column '{}' in observer create overrides", other),
        }
    }
    Ok(state)
}
```

For the code generator: iterate `cols` the same way `to_create_input` does, and for each column emit a match arm that deserializes the JSON value to the column's Rust type and converts to `BindValue`. Use the existing `bind_value_from_type` / type-mapping logic already in `gen_models.rs`.

- [ ] **Step 5: Verify db-gen compiles**

Run: `cargo check -p db-gen`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add db-gen/src/gen_models.rs
git commit -m "feat(db-gen): generate ObserverAction handling for create path"
```

---

## Task 3: Update code generator — update path

**Files:**
- Modify: `db-gen/src/gen_models.rs` (around lines 3849-3885)

- [ ] **Step 1: Change update observer from per-row to batch**

Replace the current per-row loop (lines ~3849-3884) which does:
```rust
for old_row in &__old_rows {
    let old_data = serde_json::to_value(old_row)?;
    observer.on_updating(&event, &old_data, &changes_data).await?;
}
```

With a batch call:
```rust
let old_data = serde_json::to_value(&__old_rows)?;
let event = ModelEvent { model: "...", table: "...", record_key: None };
let action = observer.on_updating(&event, &old_data, &changes_data).await?;
match action {
    ObserverAction::Prevent(err) => return Err(err),
    ObserverAction::Modify(overrides) => {
        state = Self::apply_update_overrides(state, overrides)?;
    }
    ObserverAction::Continue => {}
}
```

Note: `state` is already `let mut state = self.state;` at this point in the generated code (line ~3790).

- [ ] **Step 2: Generate `apply_update_overrides` helper**

Same pattern as `apply_create_overrides` but uses `state.assign_col()` instead of `state.set_col()`. Generate one match arm per column that can appear in updates. The generated code pattern:

```rust
fn apply_update_overrides(mut state: PatchState<'_>, overrides: serde_json::Value) -> Result<PatchState<'_>> {
    let map = overrides.as_object()
        .ok_or_else(|| anyhow::anyhow!("observer overrides must be a JSON object"))?;
    for (key, val) in map {
        match key.as_str() {
            "author_id" => {
                let v: i64 = serde_json::from_value(val.clone())?;
                state = state.assign_col("author_id", v.into());
            }
            // ... one arm per column ...
            other => anyhow::bail!("unknown column '{}' in observer update overrides", other),
        }
    }
    Ok(state)
}
```

- [ ] **Step 3: Update the `save_with_db` signature for updates**

Currently the update `save_with_db` takes `observer_changes: Option<ChangesStruct>`. This parameter is only used to pass typed changes to the observer. With batch semantics, `observer_changes` is still needed — it gets serialized to JSON for the observer's `changes` parameter. No signature change needed, but the usage inside changes from per-row to batch.

- [ ] **Step 4: Verify db-gen compiles**

Run: `cargo check -p db-gen`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add db-gen/src/gen_models.rs
git commit -m "feat(db-gen): generate ObserverAction handling for update path"
```

---

## Task 4: Update code generator — delete path

**Files:**
- Modify: `db-gen/src/gen_models.rs`

**Important:** There are **two** delete code paths in `gen_models.rs`:
1. The "delete by conditions" path (around lines 1390-1593) — used by `Model::delete()` builder
2. No second path for delete — the update path has its own observer call site but delete only has one generation function

Both the soft-delete branch and hard-delete branch share the same `on_deleting` call site (lines ~1414-1437), which runs before the soft-delete/hard-delete branching.

- [ ] **Step 1: Change delete observer from per-row to batch**

Replace the current per-row loop (lines ~1414-1437) with a batch call. This runs before the soft-delete/hard-delete branching:

```rust
let old_data = serde_json::to_value(&__old_rows)?;
let event = ModelEvent { model: "...", table: "...", record_key: None };
let action = observer.on_deleting(&event, &old_data).await?;
match action {
    ObserverAction::Prevent(err) => return Err(err),
    ObserverAction::Modify(overrides) => {
        let ids: Vec<PkType> = __old_rows.iter().map(|r| r.pk_field).collect();
        let affected = Self::convert_delete_to_update(&db, &ids, overrides).await?;
        return Ok(affected);
    }
    ObserverAction::Continue => {}
}
```

When `Modify` is returned, the delete is **short-circuited** — neither the soft-delete nor hard-delete branch runs. Instead, `convert_delete_to_update` executes an UPDATE with the override columns and returns immediately via `return Ok(affected)`. The `on_deleted` after-hooks do **NOT** fire — the row was not deleted, it was updated. This is the correct semantic: `on_deleted` should only fire when a row is actually removed.

- [ ] **Step 2: Generate `convert_delete_to_update` helper**

This function takes `target_ids` (not old_rows) and builds an `UPDATE ... SET col=val WHERE pk IN (...)` query:

```rust
async fn convert_delete_to_update<'tx>(
    db: &DbConn<'tx>,
    ids: &[PkType],
    overrides: serde_json::Value,
) -> Result<u64> {
    let map = overrides.as_object()
        .ok_or_else(|| anyhow::anyhow!("observer overrides must be a JSON object"))?;
    if map.is_empty() || ids.is_empty() { return Ok(0); }

    let mut set_parts = Vec::new();
    let mut binds: Vec<BindValue> = Vec::new();
    let mut idx = 1usize;
    for (key, val) in map {
        match key.as_str() {
            "col_name" => {
                let v: ColType = serde_json::from_value(val.clone())?;
                set_parts.push(format!("col_name = ${}", idx));
                binds.push(v.into());
                idx += 1;
            }
            // ... one arm per column (generated per-model) ...
            other => anyhow::bail!("unknown column '{}' in observer delete overrides", other),
        }
    }
    let phs: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("${}", idx + i)).collect();
    let sql = format!("UPDATE {table} SET {} WHERE {pk} IN ({})", set_parts.join(", "), phs.join(", "));
    let mut q = sqlx::query(&sql);
    for b in &binds { q = bind_query(q, b.clone()); }
    for id in ids { q = q.bind(id); }
    let res = db.execute(q).await?;
    Ok(res.rows_affected())
}
```

- [ ] **Step 3: Verify db-gen compiles**

Run: `cargo check -p db-gen`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add db-gen/src/gen_models.rs
git commit -m "feat(db-gen): generate ObserverAction handling for delete path"
```

---

## Task 5: Update snapshot test

**Files:**
- Modify: `db-gen/tests/fixtures/full_stack/expected/models/article.rs`

- [ ] **Step 1: Regenerate the expected fixture**

Run the test with `UPDATE_DB_GEN_FIXTURES` to auto-update the snapshot:

```bash
cd /Users/weiloon/Projects/personal/Rust/Rustforge
UPDATE_DB_GEN_FIXTURES=1 cargo test -p db-gen -- template_generation
```

- [ ] **Step 2: Verify the snapshot diff looks correct**

Run: `git diff db-gen/tests/fixtures/`

Check that:
- `save(self)` changed to `save(mut self)`
- `ObserverAction` import added
- `on_creating` now returns `ObserverAction` and has match block
- `on_updating` uses batch call with array, has match block
- `on_deleting` uses batch call with array, has match block
- `apply_create_overrides`, `apply_update_overrides`, `convert_delete_to_update` helpers are generated
- After-hooks (`on_created`, `on_updated`, `on_deleted`) are unchanged

- [ ] **Step 3: Run the snapshot test without UPDATE flag**

Run: `cargo test -p db-gen -- template_generation`
Expected: PASS

- [ ] **Step 4: Run all db-gen tests**

Run: `cargo test -p db-gen`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add db-gen/tests/fixtures/
git commit -m "test(db-gen): update snapshot for ObserverAction support"
```

---

## Task 6: Update Rustforge scaffold — AppModelObserver

**Files:**
- Modify: `scaffold/template/app/src/internal/observers/model.rs`

- [ ] **Step 1: Add `ObserverAction` to imports**

```rust
use core_db::common::model_observer::{ModelEvent, ModelObserver, ObserverAction};
```

- [ ] **Step 2: Update dispatch macros for before-hooks**

**Critical:** With batch semantics, `on_updating` and `on_deleting` now receive `old_data` as a JSON **array** (not a single object). The macros that decode these payloads must handle this.

**`dispatch_creating`** — no array change needed (create is always single item). Just change default arm from `Ok(())` to `Ok(ObserverAction::Continue)` and return type to `ObserverAction`.

**`dispatch_updating`** — currently decodes `$old` as a single `$row`. Must now decode as `Vec<$row>` since old_data is an array. Update the macro to `decode::<Vec<$row>>($old, ...)` and change the handler signature to accept `&[Row]`. Default arm returns `Ok(ObserverAction::Continue)`.

**`dispatch_row_hook` for `deleting`** — create a new `dispatch_before_hook` macro that decodes the payload as `Vec<$row>`, calls the handler with `&[Row]`, and returns `ObserverAction`. Default arm returns `Ok(ObserverAction::Continue)`. **Important:** the scaffold version does NOT pass `$db` (matching scaffold convention where `deleting` stubs don't take `db`). The macro signature:

```rust
macro_rules! dispatch_before_hook {
    ($event:expr, $payload:expr, $(($model:ty, $row:ty, $handler:path)),+ $(,)?) => {{
        match $event.model {
            $(
                <$model>::MODEL_KEY => {
                    let rows = decode::<Vec<$row>>($payload, stringify!($row))?;
                    $handler($event, &rows).await
                }
            )+
            _ => Ok(ObserverAction::Continue),
        }
    }};
}
```

**`dispatch_row_hook` for after-hooks** (`created`, `deleted`) — these stay unchanged (single row, returns `()`).

- [ ] **Step 3: Update `on_creating` implementation**

```rust
async fn on_creating(&self, event: &ModelEvent, new_data: &serde_json::Value) -> anyhow::Result<ObserverAction> {
    dispatch_creating!(...)  // macro now returns ObserverAction
}
```

- [ ] **Step 4: Update `on_updating` implementation**

```rust
async fn on_updating(&self, event: &ModelEvent, old_data: &serde_json::Value, changes: &serde_json::Value) -> anyhow::Result<ObserverAction> {
    dispatch_updating!(...)  // macro now returns ObserverAction
}
```

- [ ] **Step 5: Update `on_deleting` implementation**

```rust
async fn on_deleting(&self, event: &ModelEvent, old_data: &serde_json::Value) -> anyhow::Result<ObserverAction> {
    dispatch_before_hook!(...)  // returns ObserverAction
}
```

- [ ] **Step 6: Commit**

```bash
git add scaffold/template/app/src/internal/observers/model.rs
git commit -m "feat(scaffold): update AppModelObserver for ObserverAction return types"
```

---

## Task 7: Update Rustforge scaffold — model stubs

**Files:**
- Modify: all 12 files in `scaffold/template/app/src/internal/observers/models/*.rs`

- [ ] **Step 1: Update each stub file**

For each of the 12 model stub files, change the `creating`, `updating`, and `deleting` function signatures and return types.

Before (example `bank.rs`):
```rust
use core_db::common::model_observer::ModelEvent;

pub async fn creating(_event: &ModelEvent, _new_data: &BankCreate) -> anyhow::Result<()> {
    Ok(())
}
pub async fn updating(_event: &ModelEvent, _old_row: &BankRecord, _changes: &BankChanges) -> anyhow::Result<()> {
    Ok(())
}
pub async fn deleting(_event: &ModelEvent, _row: &BankRecord) -> anyhow::Result<()> {
    Ok(())
}
```

After:
```rust
use core_db::common::model_observer::{ModelEvent, ObserverAction};

pub async fn creating(_event: &ModelEvent, _new_data: &BankCreate) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
pub async fn updating(_event: &ModelEvent, _old_rows: &[BankRecord], _changes: &BankChanges) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
pub async fn deleting(_event: &ModelEvent, _rows: &[BankRecord]) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
```

**Note:** `updating` and `deleting` now take `&[Row]` (slice) instead of `&Row` because of batch semantics — the observer receives all affected rows at once. `creating` stays as single item. After-hooks (`created`, `updated`, `deleted`) remain unchanged — they return `Result<()>` and keep single-row parameters.

Files to update:
- `admin.rs`
- `bank.rs`
- `company_bank_account.rs`
- `company_crypto_account.rs`
- `content_page.rs`
- `country.rs`
- `crypto_network.rs`
- `deposit.rs`
- `introducer_change.rs`
- `user.rs`
- `user_credit_transaction.rs`
- `withdrawal.rs`

- [ ] **Step 2: Commit**

```bash
git add scaffold/template/app/src/internal/observers/models/
git commit -m "feat(scaffold): update model observer stubs for ObserverAction"
```

---

## Task 8: Rewrite mediaforge observer — single AppModelObserver

**Files:**
- Rewrite: `mediaforge/app/src/internal/observers/model.rs`

- [ ] **Step 1: Replace 3 structs with single `AppModelObserver`**

Delete `DomainModelObserver`, `AuditModelObserver`, `CompositeModelObserver<A, B>` and their `impl` blocks. Replace with:

```rust
pub struct AppModelObserver {
    db: sqlx::PgPool,
    admin_id: Option<i64>,
}

impl AppModelObserver {
    pub fn new(db: sqlx::PgPool, admin_id: Option<i64>) -> Self {
        Self { db, admin_id }
    }
}
```

- [ ] **Step 2: Implement ModelObserver for AppModelObserver**

Before-hooks return `ObserverAction`:

```rust
#[async_trait::async_trait]
impl ModelObserver for AppModelObserver {
    async fn on_creating(&self, event: &ModelEvent, new_data: &serde_json::Value) -> anyhow::Result<ObserverAction> {
        dispatch_creating!(event, new_data, ...)  // same dispatch list as current DomainModelObserver
    }

    async fn on_created(&self, event: &ModelEvent, new_data: &serde_json::Value) -> anyhow::Result<()> {
        let model_result = dispatch_row_hook!(&self.db, event, new_data, ...);
        if let Some(admin_id) = self.admin_id {
            audit::created(&self.db, admin_id, event, new_data).await?;
        }
        model_result
    }

    async fn on_updating(&self, event: &ModelEvent, old_data: &serde_json::Value, changes: &serde_json::Value) -> anyhow::Result<ObserverAction> {
        dispatch_updating!(event, old_data, changes, ...)
    }

    async fn on_updated(&self, event: &ModelEvent, old_data: &serde_json::Value, new_data: &serde_json::Value) -> anyhow::Result<()> {
        let model_result = dispatch_updated!(event, old_data, new_data, ...);
        if let Some(admin_id) = self.admin_id {
            audit::updated(&self.db, admin_id, event, old_data, new_data).await?;
        }
        model_result
    }

    async fn on_deleting(&self, event: &ModelEvent, old_data: &serde_json::Value) -> anyhow::Result<ObserverAction> {
        dispatch_before_hook!(&self.db, event, old_data, ...)
    }

    async fn on_deleted(&self, event: &ModelEvent, old_data: &serde_json::Value) -> anyhow::Result<()> {
        let model_result = dispatch_row_hook!(&self.db, event, old_data, ...);
        if let Some(admin_id) = self.admin_id {
            audit::deleted(&self.db, admin_id, event, old_data).await?;
        }
        model_result
    }
}
```

- [ ] **Step 3: Replace helper functions**

Delete `with_domain_model_observer` and `with_domain_and_audit_model_observer`. Replace with:

```rust
pub async fn with_model_observer<F, Fut, T>(db: sqlx::PgPool, admin_id: Option<i64>, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    scope_observer(Arc::new(AppModelObserver::new(db, admin_id)), f).await
}
```

- [ ] **Step 4: Update dispatch macros**

Same changes as Task 6 Step 2:

- `dispatch_creating` — return `ObserverAction`, default arm `Ok(ObserverAction::Continue)`
- `dispatch_updating` — decode `$old` as `Vec<$row>`, pass `&rows[..]` to handler, return `ObserverAction`
- Add `dispatch_before_hook` — decode payload as `Vec<$row>`, pass `&rows[..]` and `$db` to handler, return `ObserverAction`
- `dispatch_row_hook` for after-hooks — unchanged (single row, returns `()`)

Preserve the `$db` parameter in mediaforge macros (mediaforge stubs take `db: &sqlx::PgPool` for `deleting`/`deleted`/`created`).

- [ ] **Step 5: Commit**

```bash
git add mediaforge/app/src/internal/observers/model.rs
git commit -m "refactor(mediaforge): replace 3 observer structs with single AppModelObserver"
```

---

## Task 9: Update mediaforge model stubs

**Files:**
- Modify: all 12 files in `mediaforge/app/src/internal/observers/models/*.rs`

- [ ] **Step 1: Update each stub file**

Same pattern as Task 7. Change `creating`, `updating`, `deleting` return types from `anyhow::Result<()>` to `anyhow::Result<ObserverAction>`, returning `Ok(ObserverAction::Continue)`.

Note: mediaforge stubs have different signatures from Rustforge — some take `db: &sqlx::PgPool` parameter (e.g., `deleting` and `deleted` in most models, `created` in some). Preserve these extra parameters, only change the return type for before-hooks.

Example for `bank.rs`:
```rust
use core_db::common::model_observer::{ModelEvent, ObserverAction};

pub async fn creating(_event: &ModelEvent, _new_data: &BankCreate) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
pub async fn updating(_event: &ModelEvent, _old_rows: &[BankRecord], _changes: &BankChanges) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
pub async fn deleting(_db: &sqlx::PgPool, _event: &ModelEvent, _rows: &[BankRecord]) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}
```

Same batch semantics as Rustforge: `updating` and `deleting` take `&[Row]` slices. After-hooks (`created`, `updated`, `deleted`) remain unchanged — single-row, `Result<()>`.

Files to update (same 12 as Rustforge):
- `admin.rs`, `bank.rs`, `company_bank_account.rs`, `company_crypto_account.rs`
- `content_page.rs`, `country.rs`, `crypto_network.rs`, `deposit.rs`
- `introducer_change.rs`, `user.rs`, `user_credit_transaction.rs`, `withdrawal.rs`

- [ ] **Step 2: Commit**

```bash
git add mediaforge/app/src/internal/observers/models/
git commit -m "feat(mediaforge): update model observer stubs for ObserverAction"
```

---

## Task 10: Update mediaforge callers

**Files:**
- Modify: `mediaforge/app/src/internal/workflows/user_auth.rs`
- Modify: `mediaforge/app/src/internal/middleware/auth.rs`

- [ ] **Step 1: Update user_auth.rs imports and calls**

Change:
```rust
use crate::internal::observers::model::with_domain_model_observer;
```
To:
```rust
use crate::internal::observers::model::with_model_observer;
```

Update call sites:
```rust
// Line 117 — was:
let user = with_domain_model_observer(state.db.clone(), || async move { create.save().await }).await?;
// Now:
let user = with_model_observer(state.db.clone(), None, || async move { create.save().await }).await?;

// Line 388 — same pattern:
let user = with_model_observer(state.db.clone(), None, || async move { ... }).await?;
```

- [ ] **Step 2: Update middleware/auth.rs imports and calls**

Change:
```rust
use crate::internal::observers::model::with_domain_and_audit_model_observer;
```
To:
```rust
use crate::internal::observers::model::with_model_observer;
```

Update call site:
```rust
// Line 43 — was:
let response = with_domain_and_audit_model_observer(db, admin_id, || next.run(request)).await;
// Now:
let response = with_model_observer(db, Some(admin_id), || next.run(request)).await;
```

- [ ] **Step 3: Search for any other callers**

Run: `grep -r "with_domain" mediaforge/app/src/` to find any remaining references. Update them all.

- [ ] **Step 4: Verify mediaforge compiles**

Run: `cd /Users/weiloon/Projects/personal/Rust/mediaforge && cargo check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add mediaforge/app/src/internal/workflows/user_auth.rs mediaforge/app/src/internal/middleware/auth.rs
git commit -m "refactor(mediaforge): update callers to use unified with_model_observer"
```

---

## Task 11: Final verification

- [ ] **Step 1: Run full Rustforge test suite**

Run: `cd /Users/weiloon/Projects/personal/Rust/Rustforge && cargo test`
Expected: All tests PASS

- [ ] **Step 2: Run full mediaforge build**

Run: `cd /Users/weiloon/Projects/personal/Rust/mediaforge && cargo check`
Expected: PASS

- [ ] **Step 3: Verify no remaining references to deleted types**

Run: `grep -r "CompositeModelObserver\|DomainModelObserver\|AuditModelObserver\|with_domain_model_observer\|with_domain_and_audit_model_observer" mediaforge/app/src/`
Expected: No matches

**Note:** The spec mentions integration tests (observer inject on create, prevent delete, soft-delete via Modify). These require a running database and are out of scope for this plan — they should be added when the feature is exercised in a real app context. The snapshot test (Task 5) validates the generated code structure.
