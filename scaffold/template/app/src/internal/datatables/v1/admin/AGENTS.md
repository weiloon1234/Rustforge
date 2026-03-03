# Admin Datatables (v1)

Canonical admin datatable runtime implementation.

## Current Modules

1. `account.rs`
2. `http_client_log.rs`
3. `webhook_log.rs`
4. `content_page.rs`
5. `mod.rs` (SSOT catalog)

## Add New Datatable (3 edits)

Assume generated model already exists.

1. Create contract file:
   `app/src/contracts/datatable/admin/<model>.rs`
2. Create runtime hooks file:
   `app/src/internal/datatables/v1/admin/<model>.rs`
3. Add one entry in:
   `app/src/internal/datatables/v1/admin/mod.rs` (`ADMIN_SCOPED_DATATABLES`)

No direct edits needed in `internal/api/state.rs` or `internal/api/datatable.rs`.

## Contract Example

```rust
pub const SCOPED_KEY: &str = "admin.article";
pub const ROUTE_PREFIX: &str = "/datatable/article";
```

`DataTableScopedContract::scoped_key()` must return `SCOPED_KEY`.

## Runtime File Pattern

Each runtime module should expose:

1. `register_scoped(registry, db)` -> `registry.register_as(SCOPED_KEY, app_*_datatable(db))`
2. `routes(state)` -> `routes_for_scoped_contract_with_options(ROUTE_PREFIX, ...)`

Optional:

1. `scope` / `authorize` / `filter_query` / `filters` / `mappings`
2. summary helper for cross-page totals (for example account summary cards)

## Example Skeleton

```rust
pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_article_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminArticleDataTableContract,
        DataTableRouteOptions { require_bearer_auth: true },
    )
}
```

## Keep Simple

1. Prefer generated auto-filters (`f-*`) first.
2. Use `filter_query` only for non-trivial custom keys (for example `q` keyword).
3. Keep authorization logic explicit in hooks.
