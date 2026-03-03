# Datatables

Canonical datatable runtime lives under `internal/datatables/v1/admin/*`.

## Read This First

1. `internal/datatables/AGENTS.md` (this file): high-level SSOT rules.
2. `internal/datatables/v1/AGENTS.md`: version layer conventions.
3. `internal/datatables/v1/admin/AGENTS.md`: concrete admin datatable implementation examples.

## SSOT Flow

1. Contract: `app/src/contracts/datatable/admin/<model>.rs`
2. Hooks/runtime: `app/src/internal/datatables/v1/admin/<model>.rs`
3. Catalog entry: `app/src/internal/datatables/v1/admin/mod.rs` (`ADMIN_SCOPED_DATATABLES`)

Do not register datatables directly in `state.rs` or mount routes directly in `api/datatable.rs`.

## Route Pattern

Scoped routes are mounted from the catalog and stay split:

1. `POST /datatable/<scope>/query`
2. `POST /datatable/<scope>/export/csv`
3. `POST /datatable/<scope>/export/email`
4. `GET /datatable/<scope>/export/status`

No generic `/dt` route and no client-controlled model dispatch.

## Contract Constants

Each contract file must define:

1. `SCOPED_KEY`
2. `ROUTE_PREFIX`

These constants are used by runtime registration and route mounting.
