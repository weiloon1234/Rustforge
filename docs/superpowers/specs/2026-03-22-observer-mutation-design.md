# Observer Mutation Support

**Date:** 2026-03-22
**Status:** Approved

## Problem

Model observers (`ModelObserver` trait) are notification-only. All before-hooks (`on_creating`, `on_updating`, `on_deleting`) receive immutable references (`&serde_json::Value`), making it impossible to:

- Inject default values (e.g., `storage_uid`) during create
- Recalculate derived fields (e.g., profit/loss) during update
- Convert a hard delete into a soft-delete via observer logic
- Prevent an operation from proceeding

## Solution

Change before-hook return types from `Result<()>` to `Result<ObserverAction>`, where `ObserverAction` is a new enum supporting three behaviors: continue, modify, or prevent.

After-hooks (`on_created`, `on_updated`, `on_deleted`) remain unchanged — they are notification-only.

## Design

### 1. New `ObserverAction` Enum

Added to `core-db/src/common/model_observer.rs`:

```rust
/// Result of a "before" observer hook.
pub enum ObserverAction {
    /// Continue with no modifications.
    Continue,
    /// Continue but apply these field overrides before executing.
    Modify(serde_json::Value),
    /// Abort the operation with this error.
    Prevent(anyhow::Error),
}
```

### 2. Trait Signature Changes

| Hook | Current Return | New Return |
|------|---------------|------------|
| `on_creating` | `Result<()>` | `Result<ObserverAction>` |
| `on_created` | `Result<()>` | `Result<()>` (unchanged) |
| `on_updating` | `Result<()>` | `Result<ObserverAction>` |
| `on_updated` | `Result<()>` | `Result<()>` (unchanged) |
| `on_deleting` | `Result<()>` | `Result<ObserverAction>` |
| `on_deleted` | `Result<()>` | `Result<()>` (unchanged) |

Default implementations return `Ok(ObserverAction::Continue)`.

**Batch semantics:** `on_updating` and `on_deleting` are called **once per operation** (not per-row). The `old_data` parameter is a JSON **array** of all affected rows. The returned `ObserverAction` applies to the entire batch. `record_key` in `ModelEvent` is `None` for batch calls.

### 3. Generated Code Changes (db-gen/src/gen_models.rs)

**Important conventions:**
- Override keys in `Modify(json!({...}))` must be **database column names** (e.g., `storage_uid`, not `storageUid`).
- If `Modify` returns a `Value` that is not a JSON object, or contains unknown column names, the generated helper returns an error.
- Unknown keys are errors (not silently ignored) to catch typos early.

#### Create Path

The generated `save` signature changes from `save(self)` to `save(mut self)` so the state can be mutated after the observer call. After calling `on_creating`, match on the returned action:

```rust
pub async fn save(mut self) -> Result<ModelRecord> {
    // ... observer setup ...
    let action = observer.on_creating(&event, &data).await?;
    match action {
        ObserverAction::Prevent(err) => return Err(err),
        ObserverAction::Modify(overrides) => {
            self.state = Self::apply_create_overrides(self.state, overrides)?;
        }
        ObserverAction::Continue => {}
    }
    // ... proceed to save_with_db() ...
}
```

#### Update Path

The observer is called **once** with all old rows collected, not per-row. The `on_updating` hook receives `old_data` as a JSON **array** of all affected rows and the changes being applied. A single `ObserverAction` governs the entire batch:

```rust
// Collect all old rows into a JSON array
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
// ... proceed with UPDATE ...
```

Note: `apply_update_overrides` calls `state.assign_col()` (not `set_col` — `PatchState` uses `assign_col`).

#### Delete Path

Same batch approach — `on_deleting` is called **once** with `old_data` as a JSON array. When it returns `Modify`, the delete is converted into an update:

```rust
let old_data = serde_json::to_value(&__old_rows)?;
let event = ModelEvent { model: "...", table: "...", record_key: None };
let action = observer.on_deleting(&event, &old_data).await?;
match action {
    ObserverAction::Prevent(err) => return Err(err),
    ObserverAction::Modify(overrides) => {
        let affected = Self::convert_delete_to_update(&db, &target_ids, overrides).await?;
        return Ok(affected);
    }
    ObserverAction::Continue => {} // proceed with normal DELETE
}
```

#### New Per-Model Generated Helpers

Each observed model generates:

- `apply_create_overrides(state: CreateState, overrides: Value) -> Result<CreateState>` — matches JSON keys to column names, converts values to `BindValue`, calls `state.set_col()`. Returns error if `overrides` is not a JSON object or contains unknown column names.
- `apply_update_overrides(state: PatchState, overrides: Value) -> Result<PatchState>` — same for update state, calls `state.assign_col()`. Returns error if `overrides` is not a JSON object or contains unknown column names.
- `convert_delete_to_update(db: &DbConn, ids: &[PkType], overrides: Value) -> Result<u64>` — builds an `UPDATE ... SET col=val WHERE pk IN (...)` from the override columns against target IDs. Returns the number of affected rows.

These are generated per-model because each model knows its column-to-type mapping.

### 4. Observer Usage Examples

#### Inject defaults on create

```rust
async fn on_creating(&self, event: &ModelEvent, data: &Value) -> Result<ObserverAction> {
    if event.model != "media" { return Ok(ObserverAction::Continue); }

    let mut overrides = serde_json::Map::new();
    if data.get("storage_uid").map_or(true, |v| v.is_null()) {
        overrides.insert("storage_uid".into(), json!(Uuid::new_v4().to_string()));
    }

    if overrides.is_empty() {
        Ok(ObserverAction::Continue)
    } else {
        Ok(ObserverAction::Modify(Value::Object(overrides)))
    }
}
```

#### Recalculate on update

Note: `old_data` is a JSON array of all affected rows (batch semantics). `changes` is the set of columns being updated.

```rust
async fn on_updating(&self, event: &ModelEvent, old_data: &Value, changes: &Value) -> Result<ObserverAction> {
    if event.model != "order" { return Ok(ObserverAction::Continue); }

    // Use first old row to get current values for unchanged fields
    let first_row = old_data.as_array().and_then(|a| a.first());
    let price = changes.get("price")
        .or_else(|| first_row.and_then(|r| r.get("price")))
        .and_then(|v| v.as_f64()).unwrap_or(0.0);
    let qty = changes.get("quantity")
        .or_else(|| first_row.and_then(|r| r.get("quantity")))
        .and_then(|v| v.as_i64()).unwrap_or(0);

    if changes.get("price").is_some() || changes.get("quantity").is_some() {
        Ok(ObserverAction::Modify(json!({ "total": price * qty as f64 })))
    } else {
        Ok(ObserverAction::Continue)
    }
}
```

#### Prevent delete

Note: `old_data` is a JSON array. Check if *any* row matches the prevent condition.

```rust
async fn on_deleting(&self, event: &ModelEvent, old_data: &Value) -> Result<ObserverAction> {
    if event.model == "admin" {
        let rows = old_data.as_array().map(|a| a.as_slice()).unwrap_or_default();
        if rows.iter().any(|r| r.get("is_super").and_then(|v| v.as_bool()) == Some(true)) {
            return Ok(ObserverAction::Prevent(anyhow::anyhow!("cannot delete super admin")));
        }
    }
    Ok(ObserverAction::Continue)
}
```

#### Soft-delete via observer

Note: `old_data` is a JSON array but not needed here — the `Modify` overrides apply to all rows targeted by the delete. The generated `convert_delete_to_update` uses the already-collected `target_ids` to build the UPDATE.

```rust
async fn on_deleting(&self, event: &ModelEvent, _old_data: &Value) -> Result<ObserverAction> {
    if event.model == "user" {
        let now = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap();
        return Ok(ObserverAction::Modify(json!({ "deleted_at": now })));
    }
    Ok(ObserverAction::Continue)
}
```

### 5. Eliminate CompositeModelObserver (mediaforge)

Mediaforge currently has 3 observer structs (`DomainModelObserver`, `AuditModelObserver`, `CompositeModelObserver<A, B>`) which overcomplicates the design. Replace with a single `AppModelObserver` that matches Rustforge's pattern, using `Option<i64>` for `admin_id` to conditionally enable audit logging:

```rust
pub struct AppModelObserver {
    db: sqlx::PgPool,
    admin_id: Option<i64>, // None = domain logic only, Some = domain + audit
}
```

Before-hooks dispatch to model handlers and return `ObserverAction`. After-hooks dispatch to model handlers, then conditionally call audit:

```rust
async fn on_created(&self, event: &ModelEvent, new_data: &Value) -> Result<()> {
    let model_result = dispatch_row_hook!(...); // domain logic — always runs
    if let Some(admin_id) = self.admin_id {
        audit::created(&self.db, admin_id, event, new_data).await?; // audit — only if admin
    }
    model_result
}
```

Helper functions simplify to:

```rust
pub async fn with_model_observer<F, Fut, T>(db: sqlx::PgPool, admin_id: Option<i64>, f: F) -> T { ... }

// Usage:
with_model_observer(db, Some(admin_id), || { ... }).await  // admin request
with_model_observer(db, None, || { ... }).await             // non-admin request
```

Delete `DomainModelObserver`, `AuditModelObserver`, `CompositeModelObserver`, `with_domain_model_observer`, and `with_domain_and_audit_model_observer`.

## Files Changed

| File | Change |
|------|--------|
| `core-db/src/common/model_observer.rs` | Add `ObserverAction` enum, change before-hook return types |
| `db-gen/src/gen_models.rs` | Generate `ObserverAction` handling + `apply_*_overrides` + `convert_delete_to_update` per observed model |
| `db-gen/tests/fixtures/full_stack/expected/models/article.rs` | Update snapshot to match new generated code |
| `scaffold/template/.../observers/model.rs` | `Ok(())` → `Ok(ObserverAction::Continue)` for before-hooks |
| `scaffold/template/.../observers/models/*.rs` | Same return type update for 12 model stubs |
| `mediaforge/.../observers/model.rs` | Replace 3 structs with single `AppModelObserver`, delete `CompositeModelObserver` |
| `mediaforge/.../observers/models/*.rs` | Same return type update |

## Testing

1. **Snapshot test:** Update `db-gen/tests/fixtures/full_stack/expected/models/article.rs` to match new generated code
3. **Integration test:** Observer injects default on create → verify inserted row has injected value
4. **Integration test:** Observer returns `Prevent` on delete → verify row still exists
5. **Integration test:** Observer returns `Modify` on delete → verify row updated (not deleted) with override columns
