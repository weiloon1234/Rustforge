# App Backend Guide (Rust)

Scope: `app/` crate only.

Design rules:
1. Keep it simple.
2. Keep a single source of truth.
3. Do not edit generated outputs directly.
4. Do not suppress non-unused warnings; fix root causes instead.

## SSOT Files

- `models/*.rs` — schema-defined models/enums/helper methods for generated Views.
- `permissions.toml` — permission catalog.
- `settings.toml` — all configuration (auth, languages, realtime, CORS, database, etc.)
- `../i18n/*.json` — translation keys and values.

Generated outputs are produced from these inputs (plus build-time codegen) and can be overwritten.

## Warning Policy

- Allowed suppression scope is only unused-family lints:
  - `dead_code`
  - `unused_imports`
  - `unused_variables`
  - `unused_mut`
- Any other lint suppression (for example `non_camel_case_types`) is not allowed; fix the implementation instead.
- Do not use crate/dependency features that hide warnings (for example `ts-rs` `no-serde-warnings`).

## Runtime Layers

- `src/contracts/` — DTOs for API/datatable input/output.
- `src/internal/api/` — HTTP handlers and route wiring.
- `src/internal/workflows/` — business/domain logic.
- `src/internal/datatables/` — datatable runtime wiring and hooks.
- `src/internal/jobs/` — async worker jobs and schedules.
- `src/internal/middleware/` — custom request middleware.
- `src/internal/realtime/` — websocket channel policy.
- `src/validation/` — sync/async validation helpers.
- `src/seeds/` — database seeders.

Boundary rule: handlers should stay thin; workflows own domain logic.

## App State

Main state for API handlers is `src/internal/api/state.rs` (`AppApiState`).
When a shared runtime dependency is needed in handlers/datatables/workflows, add it there from boot context and pass it through state.

## Recipe: Extend Schema-Generated Model

Use this when the domain is schema-driven.

1. Add or update model/enum in `models/{domain}.rs`.
2. Add matching SQL migration in `../migrations/{timestamp}_{name}.sql`.
3. Add/adjust permissions in `permissions.toml`.
4. Add contracts in `src/contracts/api/v1/{portal}/{domain}.rs`.
5. Add workflow in `src/internal/workflows/{domain}.rs`.
6. Add API handler in `src/internal/api/v1/{portal}/{domain}.rs`.
7. Register modules in each `mod.rs` (`contracts`, `workflows`, `api`).
8. Add translation keys to all locale files in `../i18n/*.json`.
9. Run:

```bash
cargo check -p app
cargo run -p app --bin export-types
```

## Recipe: Create a New Datatable

Datatable is contract + runtime + catalog registration.

1. Contract file: `src/contracts/datatable/admin/{domain}.rs`.
2. Runtime hooks file: `src/internal/datatables/v1/admin/{domain}.rs`.
3. Register in catalog SSOT: `src/internal/datatables/v1/admin/mod.rs`.

Required contract constants:
- `SCOPED_KEY`
- `ROUTE_PREFIX`

Scoped routes expected:
- `POST /datatable/<scope>/query`
- `POST /datatable/<scope>/export/csv`
- `POST /datatable/<scope>/export/email`
- `GET /datatable/<scope>/export/status`

Do not mount per-model datatable routes manually outside the admin catalog wiring.

## Recipe: Create New API Route/Domain Flow

1. Contract types in `src/contracts/api/v1/{portal}/{domain}.rs`.
2. Workflow in `src/internal/workflows/{domain}.rs`.
3. Handler/router in `src/internal/api/v1/{portal}/{domain}.rs`.
4. Wire route in `src/internal/api/v1/{portal}/mod.rs`.
5. Wire top-level portal router in `src/internal/api/v1/mod.rs` if needed.
6. Add `mod` exports in relevant `mod.rs` files.

Validation input wrappers:
- Use `ContractJson<T>` for sync validations.
- Use `AsyncContractJson<T>` when async DB rules are involved.
- Use `Option<T>` for nullable create/full-input fields.
- Use `core_web::Patch<T>` for update fields when omitted vs `null` vs value must be different:
  - `Missing` = no change
  - `Null` = clear value
  - `Value(T)` = set/update value

## Recipe: Create Validation Rules

### Sync custom rule

- Put helper in `src/validation/{domain}.rs`.
- Return `Result<(), validator::ValidationError>`.
- Attach in contract via validation attributes.
- For PATCH inputs, prefer normalizing `Patch<String>` in the contract before validation instead of treating empty string as a nullable sentinel.

### Async/DB rule

- Prefer built-in `#[rf(async_unique/...)]` rules where possible.
- For custom async checks, implement `AsyncValidate` on the contract input type.

Register new validation modules in `src/validation/mod.rs`.

## Country Linkage Standard (`country_iso2`)

Country is framework-level reference data keyed by `countries.iso2` (string key, no numeric country ID).

For any new table that links to country:
1. Use column name `country_iso2` (not `country_id`).
2. Use type `TEXT`.
3. Add index on `country_iso2`.
4. Add DB foreign key to `countries(iso2)` by default.

SQL pattern:

```sql
country_iso2 TEXT NOT NULL,
CONSTRAINT fk_<table>_country_iso2
  FOREIGN KEY (country_iso2) REFERENCES countries(iso2),
CREATE INDEX IF NOT EXISTS idx_<table>_country_iso2 ON <table>(country_iso2);
```

Validation pattern in contracts:
1. Normalize to uppercase ISO2 (`MY`, `US`, ...).
2. Validate format is 2-letter ISO2.
3. Validate existence in `countries.iso2` (async exists check).
4. For business rules that require active countries, add `status = enabled` condition.

Frontend SSOT:
- Shared contact input uses `country_iso2: string` value shape (`frontend/src/shared/components/ContactInput.tsx`).

See also:
- `../docs/country-iso2-linkage.md` for migration-ready SQL and numeric-ID-to-ISO2 conversion steps.

## Async Domain Actions

**`tokio::spawn`** — for lightweight fire-and-forget side effects (notifications, cache invalidation, realtime broadcasts). Preferred when the work is fast and doesn't need retry/persistence.

**Job queue** — for heavy, retriable, or durable work (sending emails, processing uploads). Jobs survive server restarts and have built-in retry with backoff.

**Outbox (JobBuffer)** — for jobs that MUST be atomic with a database transaction. `JobBuffer::push()` inserts into `outbox_jobs` table within the transaction scope. The outbox sweeper flushes to Redis periodically.

**Cron scheduler** — for periodic tasks. Register in `src/internal/jobs/mod.rs` via `register_schedules()`.

- Jobs live in `src/internal/jobs/`. Register in `src/internal/jobs/mod.rs`.
- Standard dispatch: `MyJob { ... }.dispatch(&state.queue).await?`
- Transactional dispatch: `JobBuffer::new(scope.conn()).push(MyJob { ... }).await?`
- See skill `add-job` for full patterns and decision guide.

## Recipe: Realtime (WebSocket)

1. Configure channels in `settings.toml` under `[realtime.channels.*]`.
2. Implement authorization/policy in `src/internal/realtime/`.
3. Keep websocket startup in `src/bin/websocket-server.rs`.

Do not duplicate realtime access logic in unrelated layers.

## Recipe: Custom Middleware

1. Create middleware in `src/internal/middleware/{name}.rs`.
2. Register/export in `src/internal/middleware/mod.rs`.
3. Apply with route layering (`from_fn_with_state`) in the relevant API router.

Keep middleware focused: auth/context enrichment/guardrails.

## Config Guide (`settings.toml`)

All configuration lives in `settings.toml`. Env vars override via `SECTION_FIELD` convention.

Framework sections: `[app]`, `[server]`, `[database]`, `[redis]`, `[s3]`, `[auth]`, `[languages]`, `[realtime]`, `[cors]`, `[middleware]`, `[mail]`, `[worker]`, `[http_log]`.

App-specific sections: add any `[my_section]` and access via `settings.section::<T>("my_section")`.

```rust
#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
pub struct MyFeatureConfig {
    pub enabled: bool,
    pub max_retries: u32,
}

// In AppApiState::new():
let config: MyFeatureConfig = ctx.settings.section("my_feature").unwrap_or_default();
```

Wire via `AppApiState` when runtime access is needed in handlers/workflows.

## Audit Log Exclusion

Tables excluded from audit logging:
- Framework level: `FRAMEWORK_AUDIT_EXCLUDED_TABLES` in `core_db::common::model_observer` (e.g., `personal_access_tokens`)
- App level: `APP_AUDIT_EXCLUDED_TABLES` in `src/internal/observers/audit.rs` (e.g., `audit_logs`)

Add table names to the appropriate list to suppress audit entries.

## Permissions Guide (`permissions.toml`)

Permissions are SSOT and generate typed permission artifacts.

Each entry should define:
- `key`
- `guard`
- `label`
- `group`
- `description`

Admin-specific rule:
- Admin API routes are usually protected by admin auth middleware.
- Domain operations should still check domain permission keys (`*.read`, `*.manage`) explicitly where required.
- Datatable export should require `export` permission in addition to base read permission.

After updates, run generation/checks:

```bash
make gen
```

## Type Export (Rust -> TS)

For contract types used by frontend:

1. Add `#[derive(TS)]`.
2. Add `#[ts(export, export_to = "{portal}/types/")]`.
3. Shared TS types are framework-owned SSOT through `generated::ts_exports::ts_export_files()`.
   - This registry is the shared TS export registry for scaffold consumers and includes framework shapes (API/datatable/platform) plus generated enums/locales.
4. `app/src/bin/export-types.rs` orchestrates only (merge app contracts + generated shared registry, then emit files).
5. Run:

```bash
cargo run -p app --bin export-types
```

## i18n

See root `AGENTS.md` for complete i18n rules. Summary: keys are English text (fallback), `en.json` only when key ≠ value, `zh.json` needs every key. Use `core_i18n::t()` / `t_args()` in Rust.

## Seeder Recipe

1. Create seeder in `src/seeds/{name}.rs` implementing `core_db::seeder::Seeder`.
2. Register in `src/seeds/mod.rs`.
3. Run with:

```bash
./console db seed
./console db seed --name Countries          # specific seeder
./console db seed --name CountriesSeeder    # same target; `Seeder` suffix is optional
```

## Minimal Delivery Checklist

1. SSOT file updated (`schemas`/`permissions`/`configs` when relevant).
2. Migration added when schema/data changes.
3. Contracts + workflow + route wired.
4. Datatable registered (if applicable).
5. Translations added.
6. Generation/check commands pass.
