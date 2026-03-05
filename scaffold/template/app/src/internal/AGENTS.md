# Internal Layer

Single source of truth for `app/src/internal/*`.

## Boundaries

1. `api/*`: thin route handlers only.
2. `workflows/*`: business logic, DB orchestration, domain checks.
3. `datatables/*`: datatable runtime wiring and scoped hooks.
4. `jobs/*`: background job structs + registration + dispatch entrypoints.
5. `middleware/*`: app-specific middleware not already provided by framework.
6. `realtime/*`: channel authorization policy.

Keep responsibilities separate. Do not duplicate domain logic across layers.

## API Handlers

1. Parse input, call workflow, wrap response.
2. Keep handlers thin; move query/update logic into workflows.
3. Use permission wrappers (`with_permission_check_*`) on routes.
4. Use `ContractJson<T>` for sync validation and `AsyncContractJson<T>` when async rules exist.
5. Register domain routers in `internal/api/v1/mod.rs`.

## Workflows

1. One workflow module per domain.
2. Use generated model/query APIs for CRUD.
3. Keep not-found and validation errors explicit and translated.
4. Use snowflake IDs where required (`generate_snowflake_i64()`).

## Datatables

Datatable runtime is versioned and portal-scoped.

1. Contracts live in `app/src/contracts/datatable/admin/<model>.rs`.
2. Runtime hooks live in `app/src/internal/datatables/v1/admin/<model>.rs`.
3. Registry SSOT is `app/src/internal/datatables/v1/admin/mod.rs` (`ADMIN_SCOPED_DATATABLES`).

Never register datatables in API state and never mount per-model datatable routes directly in `internal/api/datatable.rs`.

Scoped routes:

1. `POST /datatable/<scope>/query`
2. `POST /datatable/<scope>/export/csv`
3. `POST /datatable/<scope>/export/email`
4. `GET /datatable/<scope>/export/status`

Add a new admin datatable with exactly 3 edits:

1. Add contract file.
2. Add runtime hooks file.
3. Register one entry in admin catalog.

Runtime module pattern:

1. Must expose `register_scoped(registry, db)`.
2. Must expose `routes(state)`.
3. Optional hooks: `scope`, `authorize`, `filter_query`, `filters`, `mappings`.

Auth requirements in hooks:

1. Query: base table permission.
2. Export: base table permission + `Permission::Export`.

Contract rules:

1. Define `SCOPED_KEY`.
2. Define `ROUTE_PREFIX`.
3. `DataTableScopedContract::scoped_key()` returns `SCOPED_KEY`.

## Jobs

1. Define jobs as serializable structs implementing `Job`.
2. Register jobs and schedules in `jobs/mod.rs`.
3. Dispatch from workflows, not from route glue.

## Middleware

1. Keep middleware focused (auth/context enrichment).
2. Apply through route layering (`from_fn_with_state`).
3. Rely on framework-provided standard stack for common concerns.

## Realtime

1. Keep channel-level access control here.
2. Align logic with `app/configs.toml` channel config.
