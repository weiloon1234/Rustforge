# Rustforge Starter

## Project Structure

```
app/                      Rust application crate
  models/                 Model/enum definitions (SSOT for code generation)
  permissions.toml        Permission catalog (SSOT)
  settings.toml           All configuration (SSOT) — env vars override via SECTION_FIELD
  src/
    bin/                  Executables: api-server, websocket-server, worker, console, export-types
    contracts/            DTOs for API and datatable input/output
      api/v1/{portal}/    Per-portal API contracts
      datatable/admin/    Admin datatable contracts
    internal/
      api/                HTTP handlers and route wiring
        state.rs          AppApiState — shared runtime state
        v1/{portal}/      Per-portal handlers
      config/             App-specific config structs (loaded via settings.section())
      datatables/         Datatable hooks and catalog registration
      jobs/               Background job definitions and scheduling
      middleware/         Custom request middleware (auth, etc.)
      observers/          Model lifecycle hooks (audit logging, side effects)
      realtime/           WebSocket channel policy
      workflows/          Business/domain logic
    seeds/                Database seeders
    validation/           Sync/async validation helpers
frontend/                 React multi-portal frontend
  src/
    shared/               Shared components, hooks, types, stores
      components/         Reusable UI: DataTable, Select, InfiniteList, TextInput, etc.
      types/              Auto-generated TS types from Rust contracts
      hooks/              Custom hooks (useDropdown, etc.)
    admin/                Admin portal (pages, stores, types)
    user/                 User portal (pages, stores, types)
generated/                Auto-generated crate (do not edit)
migrations/               SQL migration files
i18n/                     Translation files (en.json, zh.json)
docs/                     Project-specific guides and playbooks
```

## Single Source of Truth (SSOT)

- `app/models/*.rs` — model/enum definitions and generated helpers
- `app/permissions.toml` — permission catalog
- `app/settings.toml` — all configuration; env vars override via `SECTION_FIELD` convention
- `i18n/*.json` — all user-facing translations

Generated outputs are overwritten by `make gen`. Do not edit generated files directly.

## Detailed Guides

- `app/AGENTS.md` — Rust backend: contracts, workflows, datatables, handlers, jobs, middleware, observers, validation, permissions, config
- `frontend/AGENTS.md` — React frontend: portals, routing, auth, shared components, generated types, state management

## Skills (`.claude/skills/`)

| Skill | Use when... |
|-------|-------------|
| `add-model` | Adding a new database model/table |
| `add-endpoint` | Adding a new API endpoint |
| `add-datatable` | Adding a new admin datatable with frontend page |
| `add-permission` | Adding new permissions |
| `add-job` | Adding a background job |
| `add-page` | Adding a new frontend page |
| `add-config` | Adding a custom config section to settings.toml |
| `add-migration` | Creating a database migration |
| `add-seeder` | Adding a database seeder |
| `add-observer` | Adding model lifecycle hooks |
| `add-attachment` | Adding file upload attachment fields to a model |

## Commands

```bash
make dev               # Rust API + all Vite portals + auto-gen watchers
make dev-api           # Rust API only (cargo-watch)
make dev-user          # Vite user portal only
make dev-admin         # Vite admin portal only
make gen               # Rebuild generated crate + regenerate TS types
make gen-types         # Regenerate frontend TS types only
make check             # cargo check + typecheck + frontend build
make deploy            # check + tag + push (triggers CI/CD)
make build-frontend    # Production build all portals
```

## Configuration System

**Primary:** `app/settings.toml` — committed to git, all project defaults.

**Secrets:** `.env` — gitignored, per-deployment overrides only.

**Convention:** Any `[section].field` in settings.toml can be overridden by env var `SECTION_FIELD`.
Example: `[database].url` is overridden by `DATABASE_URL` env var.

**Custom sections:** Add any `[my_section]` to settings.toml, then access via:
```rust
#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
struct MyConfig { field: String }

let config: MyConfig = ctx.settings.section("my_section")?;
```

## i18n Rules

**Single source of truth for all translation rules — backend and frontend.**

### Keys are English text
The i18n key IS the English text. It serves as the automatic fallback when no translation is found.
- `t("Submit")` → displays "Submit" in English (no en.json entry needed)
- `t("Submit")` → displays the zh.json value in Chinese

### en.json
**ONLY** entries where key differs from display value:
- Enum labels: `"enum.credit_type.credit1": "Cash Point"`
- Permission labels: `"admin.read": "Read Admins"`
- Custom display: `"Adjust Credits": "User Credit Manage"`

Do NOT add redundant entries like `"Submit": "Submit"`.

### zh.json (and all non-English locales)
**EVERY** key must be present — every key needs a translation.

### All user-facing text must be translated
- Backend API responses: `core_i18n::t("message")` / `t_args("Hello :name", &[("name", value)])`
- Frontend UI: `const { t } = useTranslation(); t("Label")`
- Parameter syntax in i18n files: `:param` — framework converts to `{{param}}` for i18next

### Permission and enum keys
Non-English keys like `admin.read`, `enum.credit_type.credit1` MUST exist in both `en.json` and `zh.json`. Keep grouped together in each locale file.

## Type Generation (Rust → TypeScript)

**Prefer generated types.** Contracts with `#[derive(TS)] #[ts(export, export_to = "{portal}/types/")]` auto-generate TypeScript types via `make gen-types`.

- Import from `@{portal}/types` (e.g., `import type { DepositDatatableRow } from "@admin/types"`)
- Shared types from `@shared/types` (enums, API response shapes, datatable generics)
- Avoid custom TS interfaces when a Rust contract already generates the type
- When a datatable adds computed fields in `row_to_record`, add those fields to the contract Row struct too

## Core Rules

1. **SSOT is a must** — backend defines types, frontend consumes generated types. Never duplicate type definitions.
2. **Strongly type everything** — use enums over strings, typed IDs over raw i64, model queries over raw SQL. Raw SQL only for upserts, correlated subqueries, or cursor patterns that can't be expressed with the query builder.
3. **Handlers stay thin** — workflows own domain logic. Handlers only extract, validate, call workflow, return response.
4. **Prefer `tokio::spawn`** for async side effects over fixed worker loops when possible. Use job queues for heavy/retriable work; use `tokio::spawn` for fire-and-forget notifications, cache invalidation, etc.
5. Do not edit generated outputs directly — extend canonical definitions first, then `make gen`.
6. Do not suppress non-unused warnings — fix root causes.
