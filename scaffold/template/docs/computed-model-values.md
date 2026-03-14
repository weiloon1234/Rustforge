# Computed Model Values Guide

Use this when you need derived/read-only fields or custom methods on generated `Record` types without changing the DB schema.

## Where to implement

File: `app/models/{model_name}.rs`

- Put methods directly inside `#[rf_record_impl] impl XxxRecord`.
- Plain methods stay callable as normal generated methods.
- `#[rf_computed]` is only for methods that should also be exported into generated JSON shapes.

## Example: `UserCreditTransactionRecord`

File: `app/models/user_credit_transaction.rs`

```rust
#[rf_record_impl]
impl UserCreditTransactionRecord {
    pub fn enrich_transaction_type_explained(&mut self) {
        // custom_description -> params interpolation -> keep default
    }
}
```

## Consume in datatables

Call the generated method directly on the generated record:

```rust
fn map_row(&self, row: &mut UserCreditTransactionRecord, ..) -> anyhow::Result<()> {
    row.enrich_transaction_type_explained();
    Ok(())
}
```

## Consume in API handlers / workflows

Same pattern: call on the generated record directly.

```rust
let identity = admin_view.identity();
```

## Expose to API DTOs

Add computed field on output contracts and map from the record, or mark the method `#[rf_computed]` if it should also be emitted by generated export output.

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
cargo check -p generated
cargo check -p app
make gen-types
```
