# Rustforge

Rustforge is a framework-only Rust backend workspace.
It provides reusable infrastructure crates. Domain/application code should live in a separate starter/project repository.

## What Each Folder Does

| Folder | Purpose | Consumer-side bootstrap |
| --- | --- | --- |
| `bootstrap/` | Unified startup for web, realtime, worker, and console. Builds `BootContext` (settings, db, redis, storage, queue, mailer). Includes framework console utilities like `assets publish`. | Use `bootstrap::web::start_server`, `bootstrap::realtime::start_server`, `bootstrap::jobs::start_worker` or `start_with_context`, `bootstrap::console::start_console` in starter binaries. |
| `core-config/` | Runtime config/env loader (`Settings` and typed sub-settings). | Provide `.env` + `app/configs.toml` in starter. Set `APP_CONFIGS_PATH` if needed. |
| `core-db/` | DB infra, platform repos, migration/seeder commands, auth/platform utilities. | Run migrations/seeders from starter console (`migrate`, `db seed`, `make seeder`). Use generated/platform repos in app workflows. |
| `core-datatable/` | Generic datatable execution, filters, registry, async export manager. | Register generated datatables in starter state and mount routes via `core_web::datatable`. |
| `core-docs/` | Framework docs web UI router. | Auto-nested by `bootstrap::web::start_server` when `ENABLE_FRAMEWORK_DOCS=true`. |
| `core-http-log/` | Inbound webhook and outbound HTTP logging plus cleanup job. | Enable env flags, add `WebhookLogLayer`, and use `HttpClient` wrapper for outbound calls. |
| `core-i18n/` | Locale context, translation lookup, and locale middleware. | Keep translation files in starter `i18n/{lang}.json`; middleware is applied by `bootstrap::web::start_server`. |
| `core-jobs/` | Job trait, queue, worker runtime, cron scheduler, outbox buffer. | Define app jobs (`impl Job`), register in starter `app/src/internal/jobs/mod.rs`, run with worker bootstrap. |
| `core-mailer/` | Mail transport + queueable mail job (`SendMailJob`). | Use `ctx.mailer` in workflows/API or queue mail jobs through worker. |
| `core-notify/` | Notification channel abstraction and notifiable contracts. | Implement app notification payloads/notifiable targets, then dispatch via channel implementations. |
| `core-realtime/` | WebSocket server state, auth hooks, channel policy, presence, publisher/subscriber, durable replay. | Build websocket server in starter binary with `ws_handler` + `WsServerState`; publish events from API/worker. |
| `core-web/` | Shared web primitives: auth/authz, OpenAPI router, extractors, response/error types, rules, middleware, datatable routes. | Use `ApiRouter`, `ValidatedJson`, `ApiResponse`, auth/authz helpers in starter API modules. |
| `db-gen/` | Schema/config/permission-driven code generation for models/guards/localized/permissions/datatable stubs. | Starter `generated/build.rs` calls this crate from `app/schemas`, `app/permissions.toml`, `app/configs.toml`. |
| `scaffold/` | CLI generator for starter repository skeleton. | Run once to create new consumer project (`--output ... --force`). |
| `vendor/` | Vendored crates patched at workspace level (currently validator). | Used automatically by Cargo via `[patch.crates-io]`. |

## Consumer Bootstrap (Starter Side)

### 1. Generate starter project

```bash
# Option A: run scaffold from local Rustforge clone
cargo run --manifest-path scaffold/Cargo.toml -- --output /path/to/Rustforge-Starter

# Option B: install scaffold globally from git
cargo install --git https://github.com/weiloon1234/Rustforge.git scaffold
scaffold --output /path/to/Rustforge-Starter
```

Scaffold source of truth lives in `scaffold/template/`. Edit files there when updating starter output.

### 2. Dependency mode

Starter output uses git dependencies to Rustforge (`branch = "main"`).
For release stability, pin to a version tag in starter `Cargo.toml`.

### 3. Bootstrap runtime binaries in consumer repo

API server:

```rust
bootstrap::web::start_server(
    app::internal::api::build_router,
    |ctx| async move {
        bootstrap::jobs::start_with_context(
            ctx,
            app::internal::jobs::register_jobs,
            Some(app::internal::jobs::register_schedules),
        ).await
    },
).await
```

WebSocket server:

```rust
bootstrap::realtime::start_server(
    |_ctx| async move { Ok(axum::Router::new()) },
    |_ctx| async move { Ok(()) },
    bootstrap::realtime::RealtimeStartOptions::default(),
).await
```

Worker:

```rust
bootstrap::jobs::start_worker(
    app::internal::jobs::register_jobs,
    Some(app::internal::jobs::register_schedules),
).await
```

Console:

```rust
bootstrap::console::start_console::<ProjectCommands, fn(&mut Vec<Box<dyn core_db::seeder::Seeder>>)>(Some(register_seeders)).await
```

Static asset publish (from starter console):

```bash
./console assets publish --from frontend/dist --clean
```

### 4. Keep starter single sources of truth

- `app/configs.toml` (languages/auth/realtime static config)
- `app/schemas/*.toml` (model/enums SSOT)
- `app/permissions.toml` (permission catalog SSOT)
- `migrations/*.sql` (SQL state)
- `i18n/{lang}.json` (project-owned translation catalogs)

### 5. Generate + migrate + run

```bash
# in starter repo
cargo build -p generated
./console migrate pump
./console migrate run
./bin/api-server
```

### 6. Optional Ubuntu server installer

Starter includes an idempotent installer:

```bash
sudo ./scripts/install-ubuntu.sh
```

It can create/reuse an isolated project user, configure SSH access, update `.env`,
configure nginx + optional HTTPS, and wire Supervisor processes.

### 6. Optional framework docs mount

In consumer `.env`:

```bash
ENABLE_FRAMEWORK_DOCS=true
FRAMEWORK_DOCS_PATH=/framework-documentation
SERVER_PORT=4582
```

Then visit:

```text
http://127.0.0.1:4582/framework-documentation
```

Build framework docs frontend assets first:

```bash
# from starter repo (recommended)
make framework-docs-build
```

`core-docs` resolves assets in this order:
1. `FRAMEWORK_DOCS_DIST_DIR` (if set)
2. `PUBLIC_PATH + FRAMEWORK_DOCS_PATH` (starter default)
3. Rustforge crate-local `core-docs/frontend/dist`

## Framework Development

Run these only inside the Rustforge framework repository root:

```bash
make check
make test
make docs-build
```
