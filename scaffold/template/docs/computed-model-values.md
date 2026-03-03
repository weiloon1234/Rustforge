# Computed Model Values Guide

Use this when you need derived/read-only fields (example: `identity`) without changing DB schema.

## Where to implement

Use **`AdminView` extension methods**, not `AdminRow`.

- `AdminRow` is internal raw DB row shape (not stable for app-level semantics).
- `AdminView` is the app-facing generated model used by workflows/guards/DTO mapping.
- Safe extension point: `generated/src/extensions.rs` (not overwritten by generation).

## Scaffold example (`identity`)

File: `generated/src/extensions.rs`

```rust
pub trait AdminViewComputedExt {
    fn identity(&self) -> String;
}

impl AdminViewComputedExt for AdminView {
    fn identity(&self) -> String {
        admin_identity(
            Some(self.username.as_str()),
            Some(self.name.as_str()),
            self.email.as_deref(),
            Some(self.id),
        )
    }
}
```

Fallback helper (same file):

```rust
pub fn admin_identity(
    username: Option<&str>,
    name: Option<&str>,
    email: Option<&str>,
    id: Option<i64>,
) -> String {
    // username -> name -> email -> id
}
```

## Expose to API DTOs

Add computed field on output contracts and map from `AdminView`:

- `app/src/contracts/api/v1/admin/account.rs`
- `app/src/contracts/api/v1/admin/auth.rs`
- `app/src/internal/api/v1/admin/auth.rs`

Pattern:

```rust
identity: admin.identity(),
```

## Expose to datatable row payload (optional)

If frontend needs computed field in datatable JSON rows:

- Add field in datatable row contract:
  - `app/src/contracts/datatable/admin/account.rs`
- Add mapping injection in datatable hooks:
  - `app/src/internal/datatables/v1/admin/account.rs`

Pattern:

```rust
record.insert("identity".to_string(), serde_json::Value::String(identity));
```

This does **not** require adding a visible datatable column. UI can choose whether to use it.

## Verification

```bash
cargo check -p app
make gen-types
```
