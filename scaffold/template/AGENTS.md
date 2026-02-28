# Rustforge Project

Rust backend built on **Rustforge** (Axum + SQLx + Redis + S3). Each subfolder has its own `AGENTS.md` with domain-specific rules — read those when working in that folder.

## Tooling

**Use `rust-analyzer`** for type exploration, auto-completion, and go-to-definition. Do not guess types, fields, or method signatures — let the LSP resolve them. When unsure what fields or methods are available on a struct (e.g. `AppApiState`, `BootContext`, generated models), use go-to-definition or hover rather than assuming.

## App State

Two main context types are passed throughout the app:

- **`BootContext`** (from `bootstrap::boot`) — framework-level context available in console commands, jobs, and server startup. Key fields: `db` (PgPool), `redis` (Cache), `storage` (Arc\<dyn Storage\>), `queue` (RedisQueue), `mailer` (Arc\<Mailer\>), `settings` (Arc\<Settings\>).
- **`AppApiState`** (defined in `app/src/internal/api/state.rs`) — app-level state passed to HTTP handlers. Wraps `BootContext` fields plus app-specific resources (datatable registry, export managers, etc.). Extend this struct when adding new shared resources.

Use rust-analyzer to explore their full fields and methods — they evolve as the app grows.

## Folder Structure

```
app/
├── configs.toml              # Languages, auth guards, realtime, CORS config
├── permissions.toml          # Permission catalog
├── schemas/*.toml            # Model + enum definitions (code generation source)
└── src/
    ├── contracts/            # Request/response DTOs  ← has AGENTS.md
    ├── internal/
    │   ├── api/              # Route handlers + state ← has AGENTS.md
    │   ├── workflows/        # Business logic         ← has AGENTS.md
    │   ├── jobs/             # Background jobs        ← has AGENTS.md
    │   ├── middleware/        # Custom middleware      ← has AGENTS.md
    │   ├── datatables/       # Datatable executors    ← has AGENTS.md
    │   └── realtime/         # WebSocket policies     ← has AGENTS.md
    ├── validation/           # Validation rules       ← has AGENTS.md
    └── seeds/                # Database seeders       ← has AGENTS.md
frontend/                     # Multi-portal React + Vite + Tailwind 4 ← has AGENTS.md
generated/                    # Auto-generated — NEVER edit generated.rs
migrations/                   # SQL migration files (ordered numeric prefix)
i18n/                         # Translation JSON files
```

## Single Source of Truth (SSOT)

These files are the canonical definitions. Code is generated from them at compile time.

| File | Defines | Generated output |
|------|---------|------------------|
| `app/schemas/*.toml` | Models, enums, fields, relations | `generated/src/generated.rs` — model structs, enums, repos, query builders |
| `app/permissions.toml` | Permission keys + guards | `Permission` enum with `as_str()`, `from_str()` |
| `app/configs.toml` | Auth guards, languages, realtime channels, CORS | Typed `Settings` loaded at boot |

**Never edit `generated/src/generated.rs`** — it is overwritten every build. Put custom extensions in `generated/src/extensions.rs`.

### Schema format (`app/schemas/*.toml`)

```toml
[StatusEnum]
type = "enum"
storage = "string"
variants = ["Draft", "Published", "Archived"]

[model.article]
table = "article"
pk = "id"
pk_type = "i64"
id_strategy = "snowflake"
soft_delete = true
fields = [
  "id:i64", "title:string", "slug:string",
  "status:StatusEnum", "author_id:i64",
  "created_at:datetime", "updated_at:datetime"
]
```

Field types: `string`, `i16`, `i32`, `i64`, `f64`, `bool`, `datetime`, `hashed`, `Option<String>`, `serde_json::Value`, enum names.

### Permission format (`app/permissions.toml`)

```toml
[[permissions]]
key = "article.read"
guard = "admin"
label = "Read Articles"
group = "article"
description = "View article records."
```

Use in code: `Permission::ArticleRead.as_str()`, `Permission::from_str("article.read")`.

## Translations (i18n)

All user-facing strings **must** go through `core_i18n::t()`.

```rust
use core_i18n::t;

// Simple
t("Admin created")

// With parameters — replaces :param placeholders
use core_i18n::t_args;
t_args("Welcome :name", &[("name", &user.name)])
```

### Rules

1. **Keys are English text.** The key itself is the fallback — if no translation is found, `t()` returns the key as-is.
2. **Flat key-value JSON** — no nesting. One file per locale: `i18n/en.json`, `i18n/zh.json`, etc.
3. **`en.json` only needs entries where key differs from display text**, or where the key has `:param` placeholders. If key and value are identical (e.g. `"Admin created": "Admin created"`), **omit it from `en.json`** — the fallback already returns the key.
4. **Non-English locale files need every `t()` key** that appears in code.
5. Parameters use `:paramName` syntax in both key and all translations.

```json
// i18n/en.json — only divergent or parameterized keys
{
  "Credit 1": "Cash Point",
  "Welcome :name": "Welcome :name"
}

// i18n/zh.json — every key used in code
{
  "Article created": "文章创建成功",
  "Credit 1": "现金积分",
  "Welcome :name": "欢迎 :name"
}
```

### Where translations are used

- `ApiResponse::success(data, &t("message"))` — response messages
- `AppError::NotFound(t("Article not found"))` — error messages
- `AppError::Forbidden(t("Not allowed"))` — auth errors
- `AppError::Validation { message: t("Validation failed"), errors }` — validation wrappers

Locale is resolved per-request: `X-Locale` header > `Accept-Language` header > default locale.

## Error Handling

```rust
use core_web::error::AppError;
use core_i18n::t;

AppError::NotFound(t("Not found"))           // 404
AppError::BadRequest(t("Invalid input"))     // 400
AppError::Unauthorized(t("Bad credentials")) // 401
AppError::Forbidden(t("Not allowed"))        // 403
AppError::Validation { message: t("Validation failed"), errors }  // 422
AppError::from(anyhow_error)                 // 500
```

## Response Envelope

```rust
use core_web::response::ApiResponse;

ApiResponse::success(data, &t("OK"))       // 200
ApiResponse::created(data, &t("Created"))  // 201
```

## Console CLI (`./console`)

### Built-in Commands

| Command | Description |
|---------|-------------|
| `./console migrate run` | Apply pending SQL migrations |
| `./console migrate revert` | Revert last migration |
| `./console migrate info` | List migration status |
| `./console migrate add {name}` | Create new migration file |
| `./console migrate pump` | Generate framework internal migrations |
| `./console db seed` | Run all default seeders |
| `./console db seed --name UserSeeder` | Run a specific seeder by name |
| `./console make seeder {name}` | Generate a new seeder file |
| `./console assets publish --from dist` | Copy static assets to `PUBLIC_PATH` |
| `./console assets publish --from dist --clean` | Same, but wipe destination first |

### Custom Project Commands

Define in `app/src/bin/console.rs`. Uses Clap derive + the `ProjectCommand` trait.

```rust
use bootstrap::boot::BootContext;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Simple command with no args
    Ping,

    /// Command with args
    Demo {
        #[arg(long)]
        name: String,
    },

    /// Nested subcommand group
    #[command(subcommand)]
    Cache(CacheCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum CacheCommands {
    /// Flush application cache
    Flush,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            Self::Ping => println!("pong"),
            Self::Demo { name } => {
                println!("Hello {name} from {}", ctx.settings.app.name);
            }
            Self::Cache(CacheCommands::Flush) => {
                ctx.redis.flush().await?;
                println!("Cache flushed");
            }
        }
        Ok(())
    }
}
```

Custom commands receive `BootContext` with full access to `db`, `redis`, `storage`, `queue`, `mailer`, `settings`.

Usage: `./console ping`, `./console demo --name test`, `./console cache flush`.

## Migrations

SQL files in `migrations/` with numeric prefix. After adding a schema, write the matching migration.

```
migrations/0000000001000_admin_auth.sql
migrations/0000000002000_articles.sql
```

## Frontend (React + Vite + Tailwind 4)

The `frontend/` directory contains a multi-portal SPA setup. Each portal has its own Vite config, HTML entry, CSS theme, and source tree. See `frontend/AGENTS.md` for full details.

| Portal | URL | Dev port | Vite config | Source |
|--------|-----|----------|-------------|--------|
| user | `/` | 5173 | `vite.config.user.ts` | `src/user/` |
| admin | `/admin/` | 5174 | `vite.config.admin.ts` | `src/admin/` |

### Dev servers

```bash
make dev            # Rust API (:3000) + Vite user (:5173) + Vite admin (:5174)
make dev-api        # Rust API only
make dev-user       # Vite user only
make dev-admin      # Vite admin only
```

Both Vite dev servers proxy `/api` requests to the Rust API on `:3000`.

### Production build

```bash
make build-frontend   # Cleans public/, builds admin → public/admin/, then user → public/
```

Build order matters: admin first (into `public/admin/`), then user (into `public/` with `emptyOutDir: false`) so the user build doesn't wipe the admin output.

### Tailwind 4 — CSS-only theming

No `tailwind.config.js`. Each portal's `app.css` uses `@import "tailwindcss"` and `@theme { }` for portal-specific design tokens. The shared `postcss.config.js` just enables `@tailwindcss/postcss`.

### Production serving (Rust side)

In `app/src/internal/api/mod.rs`, `build_router` mounts:
1. `/admin/*` → `public/admin/index.html` via `nest_service` (admin SPA fallback)
2. `/*` → `public/index.html` via `static_assets_router` (user SPA fallback)

Admin is mounted first so `/admin/*` is matched before the catch-all user SPA.

### Adding a new portal

Adding a role portal (e.g. `merchant`) touches backend, frontend, and build config. Use the admin portal as the reference implementation.

#### Backend (Rust)

| # | File | What to do |
|---|------|------------|
| 1 | `app/configs.toml` | Add `[auth.guards.merchant]` with provider, TTL, refresh TTL |
| 2 | `app/schemas/merchant.toml` | Define model + enum (`auth = true`, `auth_model = "merchant"`) |
| 3 | `app/permissions.toml` | Add permission entries scoped to the new guard |
| 4 | `migrations/{next}_merchant_auth.sql` | Create table + indexes |
| 5 | `app/src/contracts/api/v1/merchant/account.rs` | CRUD DTOs (CreateInput, UpdateInput, Output) |
| 6 | `app/src/contracts/api/v1/merchant/auth.rs` | Auth DTOs (LoginInput, RefreshInput, ProfileOutput) |
| 7 | `app/src/contracts/datatable/merchant/` | Datatable query/export contracts |
| 8 | `app/src/internal/workflows/merchant.rs` | CRUD business logic |
| 9 | `app/src/internal/workflows/merchant_auth.rs` | Auth business logic (login, refresh, logout, profile) |
| 10 | `app/src/internal/middleware/auth.rs` | Add `require_merchant` guard function |
| 11 | `app/src/internal/api/v1/merchant/account.rs` | CRUD route handlers |
| 12 | `app/src/internal/api/v1/merchant/auth.rs` | Auth route handlers |
| 13 | `app/src/internal/api/v1/mod.rs` | Mount `/merchant` and `/merchant/auth` routes |
| 14 | `app/src/internal/datatables/portal/merchant/account.rs` | Datatable executor |
| 15 | `app/src/internal/api/state.rs` | Register merchant datatable in `DataTableRegistry` |
| 16 | `app/src/internal/api/mod.rs` | Add SPA serving — see below |
| 17 | `app/src/seeds/merchant_bootstrap_seeder.rs` | Bootstrap seed data |
| 18 | `i18n/*.json` | Add translation keys |
| 19 | Wire `mod` declarations | Add `pub mod` / `mod` in every relevant `mod.rs` |

**SPA serving in `build_router`** (`app/src/internal/api/mod.rs`) — add **before** the user SPA catch-all:

```rust
// Merchant SPA: /merchant/* → public/merchant/index.html
let merchant_public = public_path.join("merchant");
let merchant_index = merchant_public.join("index.html");
if merchant_public.is_dir() && merchant_index.is_file() {
    router = router.nest_service(
        "/merchant",
        ServeDir::new(&merchant_public).fallback(ServeFile::new(&merchant_index)),
    );
} else {
    router = router
        .route("/merchant", axum_get(merchant_dev))
        .route("/merchant/{*path}", axum_get(merchant_dev));
}
```

And a dev handler that serves HTML loading from the Vite dev server (copy `admin_dev`, change port and entry path):

```rust
async fn merchant_dev() -> Html<&'static str> {
    Html(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Merchant</title>
    <script type="module" src="http://localhost:5175/@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "http://localhost:5175/@react-refresh"
      RefreshRuntime.injectIntoGlobalHook(window)
      window.$RefreshReg$ = () => {}
      window.$RefreshSig$ = () => (type) => type
      window.__vite_plugin_react_preamble_installed__ = true
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://localhost:5175/src/merchant/main.tsx"></script>
  </body>
</html>"#)
}
```

#### Frontend

| # | File | What to do |
|---|------|------------|
| 1 | `frontend/vite.config.merchant.ts` | `base: "/merchant/"`, `server.port: 5175`, `outDir: "../public/merchant"` |
| 2 | `frontend/merchant.html` | Entry HTML with `<script type="module" src="/src/merchant/main.tsx">` |
| 3 | `frontend/src/merchant/main.tsx` | `<BrowserRouter basename="/merchant">` |
| 4 | `frontend/src/merchant/App.tsx` | Routes with `<ProtectedRoute>` |
| 5 | `frontend/src/merchant/app.css` | `@import "tailwindcss"` + `@theme {}` + `rf-*` component classes |
| 6 | `frontend/src/merchant/api.ts` | `createApiClient` wired to auth store |
| 7 | `frontend/src/merchant/stores/auth.ts` | `createAuthStore` with `/api/v1/merchant/auth/*` endpoints |
| 8 | `frontend/src/merchant/types/` | Generated TS types (run `make gen-types`) |
| 9 | `frontend/package.json` | Add `dev:merchant` and `build:merchant` scripts |
| 10 | `frontend/package.json` `build` script | Add `npm run build:merchant` **before** `build:user` |

#### Build & Dev

| # | File | What to do |
|---|------|------------|
| 1 | `Makefile` | Add `dev-merchant` target; add `npm --prefix frontend run dev:merchant &` to `dev` target |
| 2 | Port allocation | Pick an unused port (5175, 5176, ...) for the Vite dev server |

Build order in `package.json` `build` script matters: nested portals (`merchant`, `admin`) must build before `user`, because the user build uses `emptyOutDir: false` while nested portals use `emptyOutDir: true` within their subdirectory.

## New Feature Checklist

1. Schema → `app/schemas/{domain}.toml`
2. Migration → `migrations/{number}_{name}.sql`
3. Permissions → `app/permissions.toml`
4. Contracts → `app/src/contracts/api/v1/{portal}/{domain}.rs` (add `#[derive(TS)]` for frontend types)
5. Workflow → `app/src/internal/workflows/{domain}.rs`
6. Handler → `app/src/internal/api/v1/{portal}/{domain}.rs`
7. Wire routes → `app/src/internal/api/v1/mod.rs`
8. Module declarations → add `mod`/`pub mod` in relevant `mod.rs`
9. Translations → add keys to all `i18n/*.json` files
10. `cargo check` to trigger code generation
11. Run `make gen-types` to regenerate frontend TypeScript types from contracts
