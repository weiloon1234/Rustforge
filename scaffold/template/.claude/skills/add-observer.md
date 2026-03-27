---
name: add-observer
description: Add model lifecycle hooks for side effects and audit
---

# Add a Model Observer

Follow these steps to add lifecycle hooks (observers) for a model. Observers trigger on create, update, and delete events for side effects, validation, and audit logging.

## Step 1: Create the observer handler

Create `app/src/internal/observers/models/{model}.rs`:

```rust
use core_db::common::model_observer::{ModelEvent, ObserverAction};
use generated::models::{MyDomainCreate, MyDomainRecord, MyDomainChanges};

/// Called before a record is created. Return `ObserverAction::Abort(reason)` to prevent creation.
pub async fn creating(
    _event: &ModelEvent,
    _new_data: &MyDomainCreate,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

/// Called after a record is successfully created.
pub async fn created(
    _event: &ModelEvent,
    _row: &MyDomainRecord,
) -> anyhow::Result<()> {
    Ok(())
}

/// Called before records are updated. Return `ObserverAction::Abort(reason)` to prevent the update.
pub async fn updating(
    _event: &ModelEvent,
    _old_rows: &[MyDomainRecord],
    _changes: &MyDomainChanges,
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

/// Called after a record is successfully updated.
pub async fn updated(
    _event: &ModelEvent,
    _old_row: &MyDomainRecord,
    _new_row: &MyDomainRecord,
) -> anyhow::Result<()> {
    Ok(())
}

/// Called before records are deleted. Return `ObserverAction::Abort(reason)` to prevent deletion.
pub async fn deleting(
    _event: &ModelEvent,
    _rows: &[MyDomainRecord],
) -> anyhow::Result<ObserverAction> {
    Ok(ObserverAction::Continue)
}

/// Called after a record is successfully deleted.
pub async fn deleted(
    _event: &ModelEvent,
    _row: &MyDomainRecord,
) -> anyhow::Result<()> {
    Ok(())
}
```

Conventions:
- Import types from `generated::models::*`, NOT from `crate::gen::entities`.
- The three generated types per model are: `{Model}Create` (for creating), `{Model}Record` (for the full row), `{Model}Changes` (for updating).
- `creating` / `updating` / `deleting` are "before" hooks that can abort the operation by returning `ObserverAction::Abort(reason)`.
- `created` / `updated` / `deleted` are "after" hooks for side effects.
- The `event` parameter provides context like the current user and database connection.
- Keep observer logic lightweight -- dispatch jobs for heavy work.

## Step 2: Register in the model observer dispatcher

Update `app/src/internal/observers/model.rs` to wire the new observer.

Add the module import at the top:
```rust
use super::models::{model};
```

Add entries to each `dispatch_*!()` macro call. The macro uses **tuple syntax** with the model type, the associated type, and the handler function path:

```rust
// In the creating dispatcher:
dispatch_creating!(
    event,
    new_data,
    // ... existing entries
    (MyDomainModel, MyDomainCreate, models::my_domain::creating),
)

// In the created dispatcher:
dispatch_created!(
    event,
    row,
    // ... existing entries
    (MyDomainModel, MyDomainRecord, models::my_domain::created),
)

// In the updating dispatcher:
dispatch_updating!(
    event,
    old_rows,
    changes,
    // ... existing entries
    (MyDomainModel, MyDomainRecord, MyDomainChanges, models::my_domain::updating),
)

// In the updated dispatcher:
dispatch_updated!(
    event,
    old_row,
    new_row,
    // ... existing entries
    (MyDomainModel, MyDomainRecord, models::my_domain::updated),
)

// Repeat the same tuple pattern for deleting and deleted.
```

## Step 3: Wire the module export

Add `pub mod {model};` to `app/src/internal/observers/models/mod.rs`.

## Step 4: Audit logging

Audit logging is typically handled automatically by the `AppModelObserver` implementation. When a record is created, updated, or deleted, the observer calls `audit::created()`, `audit::updated()`, or `audit::deleted()` automatically.

No additional code is needed for basic audit logging.

## Step 5: Exclude from audit (optional)

If this model should NOT be audit-logged (e.g., high-frequency or ephemeral records), add the table name to the exclusion list.

Edit `app/src/internal/observers/audit.rs`:

```rust
pub const APP_AUDIT_EXCLUDED_TABLES: &[&str] = &[
    // ... existing exclusions
    "my_model_table_name",
];
```

## Step 6: Verify

```bash
cargo check
```

Common issues:
- Import path mismatches -- types come from `generated::models::*`.
- Missing `dispatch_*!` entries -- each lifecycle hook needs its own registration.
- Wrong tuple syntax in dispatch macros -- follow the exact pattern of existing entries.
- Using heavy logic in "before" hooks that slows down writes -- use "after" hooks + jobs instead.
- Missing mod export in `observers/models/mod.rs`.
