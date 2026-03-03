# Datatables v1

Versioned datatable runtime layer.

## Scope

Keep portal-specific runtime modules under:

1. `internal/datatables/v1/admin/*`

If another portal is added later, create:

1. `internal/datatables/v1/<portal>/*`

## SSOT Rules

1. Do not register datatables directly in API state files.
2. Do not mount per-datatable routes directly in API router files.
3. Keep registration + route mounting in one catalog file per portal:
   `internal/datatables/v1/<portal>/mod.rs`.

For current scaffold admin portal, use:

1. `internal/datatables/v1/admin/mod.rs`

## Cross-References

1. Contract DTO/metadata: `app/src/contracts/datatable/admin/*.rs`
2. API route composition: `app/src/internal/api/datatable.rs`
