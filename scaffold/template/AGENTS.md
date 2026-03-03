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
    │   ├── middleware/       # Custom middleware      ← has AGENTS.md
    │   ├── datatables/       # Datatable executors    ← has AGENTS.md
    │   └── realtime/         # WebSocket policies     ← has AGENTS.md
    ├── validation/           # Validation rules       ← has AGENTS.md
    └── seeds/                # Database seeders       ← has AGENTS.md
frontend/                     # Multi-portal React + Vite + Tailwind 4 ← has AGENTS.md
generated/                    # Auto-generated crate (do not edit generated outputs directly)
docs/                         # Focused guides (portal setup, custom commands, etc.)
migrations/                   # SQL migration files (ordered numeric prefix)
i18n/                         # Translation JSON files
```

## Single Source of Truth (SSOT)

These files are the canonical definitions. Code is generated from them at compile time.

| File | Defines | Generated output |
|------|---------|------------------|
| `app/schemas/*.toml` | Models, enums, fields, relations | `generated/src/models/*`, auth guards, and datatable skeletons |
| `app/permissions.toml` | Permission keys + guards | `Permission` enum with `as_str()`, `from_str()` |
| `app/configs.toml` | Auth guards, languages, realtime channels, CORS | Typed `Settings` + generated auth/localization artifacts |

**Never edit generated outputs directly** (for example: `generated/src/lib.rs`, `generated/src/models/*`, `generated/src/guards/*`, `generated/src/permissions.rs`, `generated/src/localized.rs`) — they are overwritten by generation/build steps. Put custom extensions in `generated/src/extensions.rs`.

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

All user-facing strings **must** go through the translation layer.

- Backend (Rust): use `core_i18n::t()` / `core_i18n::t_args()`.
- Frontend (TypeScript/React): use `t(...)` from `react-i18next`.
- Do **not** hardcode user-facing text in Rust, TS, or TSX.

```rust
use core_i18n::t;

// Simple
t("Admin created")

// With parameters — replaces :param placeholders
use core_i18n::t_args;
t_args("Welcome :name", &[("name", &user.name)])
```

```tsx
import { useTranslation } from "react-i18next";

function Header() {
  const { t } = useTranslation();
  return <h1>{t("Admin dashboard")}</h1>;
}
```

### Rules

1. **Keys are English text.** The key itself is the fallback — if no translation is found, `t()` returns the key as-is.
2. **Flat key-value JSON** — no nesting. One file per locale: `i18n/en.json`, `i18n/zh.json`, etc.
3. **`en.json` only needs entries where key differs from display text**, or where the key has `:param` placeholders. If key and value are identical (e.g. `"Admin created": "Admin created"`), **omit it from `en.json`** — the fallback already returns the key.
4. **Non-English locale files need every `t()` key** that appears in code.
5. Parameters use `:paramName` syntax in both key and all translations.
6. Frontend labels/buttons/placeholders/table headers/empty states/toasts/modal text must use `t(...)`.
7. Allowed hardcoded strings: internal debug logs, telemetry keys, and non-user-facing constants only.

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
- React JSX labels/buttons/help text: `t("...")`
- Frontend toast/dialog messages: `t("...")` unless the backend already returned a localized message

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

Keep root guidance small. For full custom command patterns (Clap enums, nested subcommands, `ProjectCommand` trait, and examples), open:

- `docs/custom-project-commands.md`

### Computed Model Values

Keep root guidance small. For computed/read-only model fields (for example `identity`) and the correct extension point (`AdminView`, not `AdminRow`), open:

- `docs/computed-model-values.md`

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

Keep this file lean. For the full backend + frontend + build playbook, open:

- `docs/add-new-portal.md` (primary guide)
- `frontend/AGENTS.md` (frontend-only implementation details)

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
