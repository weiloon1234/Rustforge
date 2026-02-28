# Datatables

Server-side datatable executors. Generated stubs come from `db-gen`; custom datatables are registered manually in `state.rs`.

## Custom Datatable

Override or extend generated datatables here. Registration happens in `AppApiState::new()`:

```rust
datatable_registry.register_as("article.list", custom_article_datatable(ctx.db.clone()));
```

## Datatable Contract

Define query/export contracts in `contracts/datatable/{domain}/`. They specify filters, columns, and export formats available to the datatable.
