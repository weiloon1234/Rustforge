# Rustforge Starter

Full-stack application skeleton: **Rust API** (Axum + SQLx + Redis) + **React frontend** (Vite + Tailwind 4).
Build your product here. Keep framework changes in [Rustforge](https://github.com/weiloon1234/Rustforge.git), keep domain logic in this repo.

## Prerequisites

- **Rust** (stable) + `cargo-watch` (`cargo install cargo-watch`)
- **Node.js** (20+) + npm
- **PostgreSQL** (15+)
- **Redis** (7+)

## Quick Start

```bash
# 1. Environment
cp .env.example .env          # edit DB/Redis credentials

# 2. Code generation
cargo build -p generated

# 3. Database
./console migrate pump        # generate framework migrations
./console migrate run         # apply all migrations

# 4. Frontend
make install-frontend         # npm install

# 5. Run everything
make dev                      # Rust API (:3000) + user portal (:5173) + admin portal (:5174)
```

Open [http://localhost:5173](http://localhost:5173) for the user portal and [http://localhost:5174](http://localhost:5174) for the admin portal during development.

## Repository Layout

```
app/                    Rust application crate (API, websocket, worker, console)
  configs.toml          Languages, auth guards, realtime, CORS
  permissions.toml      Permission catalog
  schemas/*.toml        Model definitions (code generation source)
  src/
    internal/api/       Route handlers + state
    internal/workflows/ Business logic
    internal/jobs/      Background jobs
    contracts/          Request/response DTOs
    validation/         Validation rules
    seeds/              Database seeders
frontend/               Multi-portal React + Vite + Tailwind 4
  src/user/             User portal (served at /)
  src/admin/            Admin portal (served at /admin/)
  src/shared/           Shared components & utilities
generated/              Auto-generated code — never edit directly
migrations/             SQL migration files
i18n/                   Translation catalogs (en.json, zh.json, ...)
public/                 Built frontend output (git-ignored)
bin/                    Shell wrappers for running services
scripts/                Server install & update scripts
```

## Development

### Make targets

| Command | What it does |
|---------|-------------|
| `make dev` | Start Rust API + both Vite portals (all-in-one) |
| `make dev-api` | Rust API only with cargo-watch (auto-reload) |
| `make dev-frontend` | Both Vite portals |
| `make dev-user` | Vite user portal only (port 5173) |
| `make dev-admin` | Vite admin portal only (port 5174) |
| `make build-frontend` | Production build all portals into `public/` |
| `make install-frontend` | `npm install` for frontend |
| `make check` | `cargo check --workspace` |
| `make gen` | Rebuild generated code |
| `make run-api` | Run API server (release) |
| `make run-ws` | Run WebSocket server |
| `make run-worker` | Run background worker |

### Frontend architecture

The frontend ships two independent SPA portals, each with its own Vite config, dev server, and CSS theme:

| Portal | URL | Dev port | Vite config | Source |
|--------|-----|----------|-------------|--------|
| User | `/` | 5173 | `vite.config.user.ts` | `frontend/src/user/` |
| Admin | `/admin/` | 5174 | `vite.config.admin.ts` | `frontend/src/admin/` |

Both dev servers proxy `/api` to the Rust API on port 3000.

**Tailwind 4**: No `tailwind.config.js` needed. Each portal customises design tokens via `@theme { }` in its own `app.css`.

**Production build**: `make build-frontend` cleans `public/`, builds admin into `public/admin/`, then user into `public/`. The Rust API serves `public/admin/index.html` as the admin SPA fallback and `public/index.html` as the user SPA fallback.

### Migrations

```bash
./console migrate pump          # generate framework migrations
./console migrate run           # apply pending migrations
./console migrate revert        # revert last migration
./console migrate add my_table  # create new migration file
```

### Seeds

```bash
./console db seed                         # run all seeders
./console db seed --name AdminBootstrap   # run a specific seeder
```

## Production Deployment

### Ubuntu Server Install

```bash
sudo ./scripts/install-ubuntu.sh   # or: make server-install
```

Idempotent installer that configures: isolated Linux user, SSH access, `.env` values, nginx, Supervisor programs, and optional Let's Encrypt certificates.

### Updates

```bash
./scripts/update.sh                       # or: make server-update
RUN_MIGRATIONS=false ./scripts/update.sh  # skip migrations
```

Pulls latest code, compiles release binaries, builds frontend, runs migrations, and restarts Supervisor programs.

## Key Concepts

### Code Generation (SSOT)

| Source file | Generates |
|-------------|-----------|
| `app/schemas/*.toml` | Model structs, enums, repos, query builders |
| `app/permissions.toml` | `Permission` enum |
| `app/configs.toml` | Typed `Settings` |

Never edit `generated/src/generated.rs` — it's overwritten on every build. Put extensions in `generated/src/extensions.rs`.

### i18n

All user-facing strings go through `core_i18n::t()`. Translation files live in `i18n/`. Locale is resolved per-request from `X-Locale` or `Accept-Language` headers.

### Redis

`REDIS_CACHE_PREFIX` auto-derives from `{APP_NAME}_{APP_ENV}`. Leave empty unless you need custom namespacing.

### Dependency Pinning

This starter uses git dependencies to Rustforge `main` branch. For production stability, pin to a specific tag in `Cargo.toml`.

### Framework Documentation

```bash
make framework-docs-build
```

Publishes framework docs to `public/framework-documentation/`.
