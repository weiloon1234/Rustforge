# Computed Model Values Guide

Use this when you need derived/read-only fields or custom methods on generated `View` types without changing the DB schema.

## Where to implement

File: `app/src/internal/extensions/{model_name}.rs`

- Extend `XxxView` (the app-facing generated model), not `XxxRow` (internal raw DB shape).
- Define an extension trait in your file and implement it on the generated View type.
- Register each file in `app/src/internal/extensions/mod.rs`.

## Example: `UserCreditTransactionViewExt`

File: `app/src/internal/extensions/user_credit_transaction.rs`

```rust
use generated::models::UserCreditTransactionView;

pub trait UserCreditTransactionViewExt {
    fn enrich_transaction_type_explained(&mut self);
}

impl UserCreditTransactionViewExt for UserCreditTransactionView {
    fn enrich_transaction_type_explained(&mut self) {
        // custom_description → params interpolation → keep default
    }
}
```

## Consume in datatables

Import the trait and call on the `WithRelations` row (which `DerefMut`s to `View`):

```rust
use crate::internal::extensions::user_credit_transaction::UserCreditTransactionViewExt;

fn map_row(&self, row: &mut UserCreditTransactionWithRelations, ..) -> anyhow::Result<()> {
    row.enrich_transaction_type_explained();
    Ok(())
}
```

## Consume in API handlers / workflows

Same pattern — import the trait, call on the View:

```rust
use crate::internal::extensions::admin::AdminViewExt;

let identity = admin_view.identity();
```

## Expose to API DTOs

Add computed field on output contracts and map from the View:

```rust
identity: admin.identity(),
```

## Expose to datatable row payload (optional)

If frontend needs computed field in datatable JSON rows:

- Add field in datatable row contract (`app/src/contracts/datatable/admin/{model}.rs`)
- Map in `row_to_record` hook (`app/src/internal/datatables/v1/admin/{model}.rs`)

```rust
record.insert("identity".to_string(), serde_json::Value::String(identity));
```

This does **not** require adding a visible datatable column. UI can choose whether to use it.

## Verification

```bash
cargo check -p app
make gen-types
```
