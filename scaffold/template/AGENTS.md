# Rustforge Project

This is the scaffold root instruction index.

## Instruction Files

- `app/AGENTS.md` — Rust backend implementation guide (contracts, workflows, datatables, jobs, middleware, realtime, validation, permissions, configs).
- `frontend/AGENTS.md` — Frontend implementation guide (portals, routing, auth store, shared components, generated TS types).

## Single Source of Truth

- `app/models/*.rs` — model/enum definitions and generated View helpers.
- `app/permissions.toml` — permission catalog.
- `app/configs.toml` — auth/languages/realtime/CORS config.
- `i18n/*.json` — all user-facing translations.

Generated outputs are overwritten by generation/build. Do not edit generated files directly.

## Project Map

- `app/` — Rust application crate.
- `frontend/` — React portals.
- `generated/` — generated crate.
- `migrations/` — SQL migrations.
- `docs/` — focused guides.

## Common Commands

```bash
make dev
make dev-api
make dev-user
make dev-admin
make build-frontend
make gen
make gen-types
```

## Rule

Keep implementations simple and SSOT-driven. Extend canonical definitions first, then regenerate artifacts.
