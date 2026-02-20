pub const ROOT_CARGO_TOML: &str = r#"[workspace]
resolver = "2"
members = ["app", "generated"]

[workspace.package]
edition = "2021"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["macros"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "macros", "time", "uuid", "json"] }
validator = { version = "0.20", features = ["derive"] }
schemars = { version = "0.8", features = ["chrono", "uuid1"] }
async-trait = "0.1"
clap = { version = "4", features = ["derive"] }
toml = "0.9"
uuid = { version = "1", features = ["serde", "v4"] }
time = { version = "0.3", features = ["serde"] }

bootstrap = { path = "__RUSTFORGE_PATH__/bootstrap" }
core-config = { path = "__RUSTFORGE_PATH__/core-config" }
core-db = { path = "__RUSTFORGE_PATH__/core-db" }
core-datatable = { path = "__RUSTFORGE_PATH__/core-datatable" }
core-i18n = { path = "__RUSTFORGE_PATH__/core-i18n" }
core-jobs = { path = "__RUSTFORGE_PATH__/core-jobs" }
core-notify = { path = "__RUSTFORGE_PATH__/core-notify" }
core-realtime = { path = "__RUSTFORGE_PATH__/core-realtime" }
core-web = { path = "__RUSTFORGE_PATH__/core-web" }
db-gen = { path = "__RUSTFORGE_PATH__/db-gen" }
"#;

pub const ROOT_ENV_EXAMPLE: &str = r#"APP_NAME=starter
APP_ENV=local
APP_KEY=dev-only
APP_TIMEZONE=+08:00

# Starter-owned translation catalogs.
I18N_DIR=i18n

ENABLE_FRAMEWORK_DOCS=true
FRAMEWORK_DOCS_PATH=/framework-documentation
ENABLE_OPENAPI_DOCS=true
OPENAPI_DOCS_PATH=/openapi
OPENAPI_JSON_PATH=/openapi.json

APP_CONFIGS_PATH=app/configs.toml
APP_MIGRATIONS_DIR=migrations
APP_SEEDERS_DIR=app/src/seeds
PUBLIC_PATH=public

SERVER_HOST=127.0.0.1
SERVER_PORT=3000

REALTIME_ENABLED=true
REALTIME_HOST=127.0.0.1
REALTIME_PORT=3010
REALTIME_REQUIRE_AUTH=false

DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/starter
REDIS_URL=redis://127.0.0.1:6379/0
# Optional override. Leave empty to auto-derive "{APP_NAME}_{APP_ENV}".
REDIS_CACHE_PREFIX=

RUN_WORKER=false
WORKER_CONCURRENCY=10
WORKER_SWEEP_INTERVAL=30

SEED_ADMIN_BOOTSTRAP_IN_PROD=false
SEED_ADMIN_DEVELOPER_EMAIL=developer@example.com
SEED_ADMIN_DEVELOPER_PASSWORD=password123
SEED_ADMIN_SUPERADMIN_EMAIL=superadmin@example.com
SEED_ADMIN_SUPERADMIN_PASSWORD=password123
"#;

pub const ROOT_MAKEFILE: &str = r#"SHELL := /bin/bash
RUSTFORGE_PATH ?= __RUSTFORGE_PATH__

.PHONY: help
help:
	@echo "Starter Makefile"
	@echo "--------------"
	@echo "  make run-api"
	@echo "  make run-ws"
	@echo "  make run-worker"
	@echo "  make console CMD='route list'"
	@echo "  make route-list"
	@echo "  make migrate-pump"
	@echo "  make migrate-run"
	@echo "  make assets-publish ASSETS_ARGS='--from frontend/dist --clean'"
	@echo "  make framework-docs-build"
	@echo "  make check"
	@echo "  make gen"

.PHONY: run-api
run-api:
	./bin/api-server

.PHONY: run-ws
run-ws:
	./bin/websocket-server

.PHONY: run-worker
run-worker:
	./bin/worker

.PHONY: console
console:
	./bin/console $(CMD)

.PHONY: route-list
route-list:
	./bin/console route list

.PHONY: migrate-pump
migrate-pump:
	./bin/console migrate pump

.PHONY: migrate-run
migrate-run:
	./bin/console migrate run

.PHONY: assets-publish
assets-publish:
	./bin/console assets publish $(ASSETS_ARGS)

.PHONY: framework-docs-build
framework-docs-build:
	npm --prefix $(RUSTFORGE_PATH)/core-docs/frontend run build

.PHONY: check
check:
	cargo check --workspace

.PHONY: gen
gen:
	cargo build -p generated
"#;

pub const ROOT_README_MD: &str = r#"# Rustforge Starter

Rustforge-Starter is the consumer application skeleton that depends on Rustforge framework crates.
Use this repository to build real products. Keep framework changes in Rustforge, keep domain logic here.

## Repository Layout

| Folder | Purpose |
| --- | --- |
| `app/` | Main application crate (API/websocket/worker/console binaries, internal modules, contracts, validation, seeds). |
| `generated/` | Generated crate from `db-gen` using `app/schemas`, `app/permissions.toml`, `app/configs.toml`. |
| `migrations/` | Application SQL migrations. |
| `i18n/` | Project-owned translation catalogs (`en.json`, `zh.json`, ...). |
| `public/` | Optional static output directory for built frontend assets (`PUBLIC_PATH`). |
| `bin/` | Short wrappers to run API/websocket/worker/console with expected env defaults. |
| `.env.example` | Runtime environment template. |
| `Cargo.toml` | Workspace root and Rustforge dependency wiring. |

## First Boot

1. Copy env and adjust values:

```bash
cp .env.example .env
```

2. Ensure PostgreSQL and Redis are running.
3. Generate code:

```bash
cargo build -p generated
```

4. Build migration files and run them:

```bash
./bin/console migrate pump
./bin/console migrate run
```

5. Start services:

```bash
./bin/api-server
./bin/websocket-server
./bin/worker
```

## i18n Ownership

This starter owns translation files.
`I18N_DIR=i18n` is set in `.env.example`, and API locale is resolved from `Accept-Language`/`x-locale` by framework middleware.

## Static Assets (Optional)

1. Keep `PUBLIC_PATH=public` (or set your own path in `.env`).
2. Build your frontend project (for example Vite `dist` output).
3. Publish files into `PUBLIC_PATH`:

```bash
./bin/console assets publish --from frontend/dist --clean
```

When `PUBLIC_PATH/index.html` exists, API server serves that folder at `/` with SPA fallback.

## Redis Key Isolation

Keep `REDIS_CACHE_PREFIX` empty by default. Framework auto-derives `{APP_NAME}_{APP_ENV}` to namespace keys.
Set `REDIS_CACHE_PREFIX` only when you need a custom prefix strategy.

## Dependency Mode

This starter uses local path dependencies to sibling Rustforge crates.
When Rustforge is published/tagged, you can switch to git dependencies in `Cargo.toml`.
"#;

pub const ROOT_I18N_EN_JSON: &str = r#"{
  "Profile updated successfully": "Profile updated successfully",
  "Invalid credentials": "Invalid credentials",
  "Access denied": "Access denied"
}
"#;

pub const ROOT_I18N_ZH_JSON: &str = r#"{
  "Profile updated successfully": "个人资料更新成功",
  "Invalid credentials": "凭证无效",
  "Access denied": "拒绝访问"
}
"#;

pub const BIN_API_SERVER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
export APP_CONFIGS_PATH="${APP_CONFIGS_PATH:-app/configs.toml}"
export APP_SEEDERS_DIR="${APP_SEEDERS_DIR:-app/src/seeds}"
export PUBLIC_PATH="${PUBLIC_PATH:-public}"
cargo run -p app --bin api-server
"#;

pub const BIN_WEBSOCKET_SERVER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
export APP_CONFIGS_PATH="${APP_CONFIGS_PATH:-app/configs.toml}"
export APP_SEEDERS_DIR="${APP_SEEDERS_DIR:-app/src/seeds}"
export PUBLIC_PATH="${PUBLIC_PATH:-public}"
cargo run -p app --bin websocket-server
"#;

pub const BIN_WORKER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
export APP_CONFIGS_PATH="${APP_CONFIGS_PATH:-app/configs.toml}"
export APP_SEEDERS_DIR="${APP_SEEDERS_DIR:-app/src/seeds}"
export PUBLIC_PATH="${PUBLIC_PATH:-public}"
cargo run -p app --bin worker
"#;

pub const BIN_CONSOLE: &str = r#"#!/usr/bin/env bash
set -euo pipefail
export APP_CONFIGS_PATH="${APP_CONFIGS_PATH:-app/configs.toml}"
export APP_SEEDERS_DIR="${APP_SEEDERS_DIR:-app/src/seeds}"
export PUBLIC_PATH="${PUBLIC_PATH:-public}"
cargo run -p app --bin console -- "$@"
"#;

pub const MIGRATIONS_GITKEEP: &str = "";
pub const PUBLIC_GITKEEP: &str = "";

pub const APP_CONFIGS_TOML: &str = r#"[languages]
default = "en"
supported = ["en", "zh"]
timezone = "+08:00"

[auth]
default = "admin"

[auth.guards.admin]
provider = "admin"
ttl_min = 120
refresh_ttl_days = 30

[realtime.channels.public]
enabled = true
guard = ""
presence_enabled = false
"#;

pub const APP_PERMISSIONS_TOML: &str = r#"# Permission catalog (single source of truth).
# Keep empty if your project does not need permissions yet.
#
# Example:
# [[permissions]]
# key = "article.manage"
# guard = "admin"
# label = "Manage Articles"
# group = "article"
# description = "Create/update/delete article resources."
"#;

pub const APP_SCHEMA_ADMIN_TOML: &str = r#"[AdminType]
type = "enum"
storage = "string"
variants = ["Developer", "SuperAdmin", "Admin"]

auth = true
auth_model = "admin"

[model.admin]
table = "admin"
pk = "id"
pk_type = "uuid"
id_strategy = "manual"
fields = [
  "id:uuid",
  "email:string",
  "password:hashed",
  "name:string",
  "admin_type:AdminType",
  "created_at:datetime",
  "updated_at:datetime"
]
"#;

pub const MIGRATION_ADMIN_AUTH_SQL: &str = r#"CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS admin (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    admin_type TEXT NOT NULL CHECK (admin_type IN ('developer', 'superadmin', 'admin')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

pub const APP_CARGO_TOML: &str = r#"[package]
name = "app"
version = "0.1.0"
edition.workspace = true

[dependencies]
bootstrap = { workspace = true }
core-config = { workspace = true }
core-db = { workspace = true }
core-datatable = { workspace = true }
core-i18n = { workspace = true }
core-jobs = { workspace = true }
core-notify = { workspace = true }
core-realtime = { workspace = true }
core-web = { workspace = true }

generated = { path = "../generated" }

anyhow = { workspace = true }
tokio = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
validator = { workspace = true }
schemars = { workspace = true }
async-trait = { workspace = true }
clap = { workspace = true }
uuid = { workspace = true }
"#;

pub const APP_LIB_RS: &str = r#"pub mod contracts;
pub mod internal;
pub mod seeds;
pub mod validation;
"#;

pub const APP_CONTRACTS_MOD_RS: &str = r#"pub mod api;
"#;

pub const APP_CONTRACTS_API_MOD_RS: &str = r#"pub mod v1;
"#;

pub const APP_CONTRACTS_API_V1_MOD_RS: &str = r#"pub mod admin_auth;
"#;

pub const APP_CONTRACTS_API_V1_ADMIN_AUTH_RS: &str = r#"use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminLoginInput {
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: String,

    #[validate(length(min = 8, max = 128))]
    #[schemars(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminLoginOutput {
    pub access_token: String,
}
"#;

pub const APP_VALIDATION_MOD_RS: &str = r#"pub mod db;
pub mod sync;
"#;

pub const APP_VALIDATION_SYNC_RS: &str = r#"use std::borrow::Cow;
use validator::ValidationError;

fn err(code: &'static str, msg: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(msg))
}

pub fn required_trimmed(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value).map_err(|_| err("required", "This field is required."))
}

pub fn alpha_dash(value: &str) -> Result<(), ValidationError> {
    core_web::rules::alpha_dash(value)
}
"#;

pub const APP_VALIDATION_DB_RS: &str = r#"use anyhow::Result;
use core_web::rules::{AsyncRule, Exists, NotExists, Unique};

pub async fn ensure_unique(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    Unique::new(table, column, value).check(db).await
}

pub async fn ensure_exists(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    Exists::new(table, column, value).check(db).await
}

pub async fn ensure_not_exists(
    db: &sqlx::PgPool,
    table: &'static str,
    column: &'static str,
    value: impl ToString,
) -> Result<bool> {
    NotExists::new(table, column, value).check(db).await
}
"#;

pub const APP_INTERNAL_MOD_RS: &str = r#"pub mod api;
pub mod datatables;
pub mod jobs;
pub mod middleware;
pub mod realtime;
pub mod workflows;
"#;

pub const APP_INTERNAL_API_MOD_RS: &str = r#"pub mod datatable;
pub mod state;
pub mod v1;

use std::sync::Arc;

use axum::{routing::get as axum_get, Json, Router};
use bootstrap::boot::BootContext;
use core_web::openapi::{
    aide::{
        openapi::{Info, OpenApi},
    },
    ApiRouter,
};

use state::AppApiState;

pub async fn build_router(ctx: BootContext) -> anyhow::Result<Router> {
    let app_state = AppApiState::new(&ctx)?;

    let api_router = ApiRouter::new().nest("/api/v1", v1::router(app_state));

    let mut api = OpenApi::default();
    api.info = Info {
        title: "starter-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ..Default::default()
    };

    let mut router =
        api_router.finish_api_with(&mut api, core_web::openapi::with_bearer_auth_scheme);

    if ctx.settings.app.enable_openapi_docs {
        let openapi_json_path = ctx.settings.app.openapi_json_path.clone();
        let openapi = Arc::new(api);

        router = router.route(
            openapi_json_path.as_str(),
            axum_get({
                let openapi = openapi.clone();
                move || {
                    let openapi = openapi.clone();
                    async move { Json((*openapi).clone()) }
                }
            }),
        );
    }

    let public_path = core_web::static_assets::public_path_from_env();
    if let Some(static_router) = core_web::static_assets::static_assets_router(&public_path) {
        router = router.merge(static_router);
    } else {
        router = router.route("/", axum_get(root));
    }

    Ok(router)
}

async fn root() -> &'static str {
    "ok"
}
"#;

pub const APP_INTERNAL_API_STATE_RS: &str = r#"use std::sync::Arc;

use bootstrap::boot::BootContext;
use core_config::DataTableUnknownFilterMode as ConfigUnknownFilterMode;
use core_datatable::{DataTableAsyncExportManager, DataTableRegistry, DataTableUnknownFilterMode};

#[derive(Clone)]
pub struct AppApiState {
    pub db: sqlx::PgPool,
    pub datatable_registry: Arc<DataTableRegistry>,
    pub datatable_async_exports: Arc<DataTableAsyncExportManager>,
    pub datatable_default_per_page: i64,
    pub datatable_unknown_filter_mode: DataTableUnknownFilterMode,
    pub app_timezone: String,
}

impl AppApiState {
    pub fn new(ctx: &BootContext) -> anyhow::Result<Self> {
        let mut datatable_registry = DataTableRegistry::new();
        crate::internal::datatables::register_all_generated_datatables(&mut datatable_registry, &ctx.db);

        let datatable_registry = Arc::new(datatable_registry);
        let datatable_async_exports =
            Arc::new(DataTableAsyncExportManager::new(datatable_registry.clone()));

        Ok(Self {
            db: ctx.db.clone(),
            datatable_registry,
            datatable_async_exports,
            datatable_default_per_page: ctx.settings.app.default_per_page as i64,
            datatable_unknown_filter_mode: map_unknown_filter_mode(
                ctx.settings.app.datatable_unknown_filter_mode,
            ),
            app_timezone: ctx.settings.i18n.default_timezone_str.clone(),
        })
    }
}

impl core_web::auth::AuthState for AppApiState {
    fn auth_db(&self) -> &sqlx::PgPool {
        &self.db
    }
}

fn map_unknown_filter_mode(mode: ConfigUnknownFilterMode) -> DataTableUnknownFilterMode {
    match mode {
        ConfigUnknownFilterMode::Ignore => DataTableUnknownFilterMode::Ignore,
        ConfigUnknownFilterMode::Warn => DataTableUnknownFilterMode::Warn,
        ConfigUnknownFilterMode::Error => DataTableUnknownFilterMode::Error,
    }
}
"#;

pub const APP_INTERNAL_API_DATATABLE_RS: &str = r#"use std::sync::Arc;

use async_trait::async_trait;
use axum::http::HeaderMap;
use core_datatable::{DataTableAsyncExportManager, DataTableContext, DataTableRegistry};
use core_web::datatable::{DataTableRouteOptions, DataTableRouteState};
use core_web::openapi::ApiRouter;

use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    core_web::datatable::routes_with_prefix_and_options(
        "/datatable",
        state,
        DataTableRouteOptions {
            include_multipart_endpoints: true,
            require_bearer_auth: true,
        },
    )
}

#[async_trait]
impl DataTableRouteState for AppApiState {
    fn datatable_registry(&self) -> &Arc<DataTableRegistry> {
        &self.datatable_registry
    }

    fn datatable_async_exports(&self) -> &Arc<DataTableAsyncExportManager> {
        &self.datatable_async_exports
    }

    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext {
        DataTableContext {
            default_per_page: self.datatable_default_per_page,
            app_timezone: self.app_timezone.clone(),
            user_timezone: core_web::utils::datatable::parse_timezone_from_headers(headers),
            actor: None,
            unknown_filter_mode: self.datatable_unknown_filter_mode,
        }
    }
}
"#;

pub const APP_INTERNAL_API_V1_MOD_RS: &str = r#"use axum::middleware::from_fn_with_state;
use core_web::openapi::{
    aide::axum::routing::get,
    ApiRouter,
};

use crate::internal::api::{datatable, state::AppApiState};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user_router())
        .nest("/admin", admin_router(state))
}

fn user_router() -> ApiRouter {
    ApiRouter::new().api_route("/health", get(user_health))
}

fn admin_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route("/health", get(admin_health))
        .merge(datatable::router(state.clone()))
        .layer(from_fn_with_state(
            state,
            crate::internal::middleware::auth::require_admin,
        ))
}

async fn user_health() -> &'static str {
    "ok"
}

async fn admin_health() -> &'static str {
    "ok"
}
"#;

pub const APP_INTERNAL_MIDDLEWARE_MOD_RS: &str = r#"pub mod auth;
"#;

pub const APP_INTERNAL_MIDDLEWARE_AUTH_RS: &str = r#"use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use core_web::error::AppError;
use generated::guards::AdminGuard;

use crate::internal::api::state::AppApiState;

pub async fn require_admin(
    state: State<AppApiState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    core_web::auth::require_auth::<AdminGuard, AppApiState>(state, request, next).await
}
"#;

pub const APP_INTERNAL_WORKFLOWS_MOD_RS: &str = r#"// Put domain workflows here.
"#;

pub const APP_INTERNAL_REALTIME_MOD_RS: &str = r#"// Put realtime channel policies/authorizers here.
"#;

pub const APP_INTERNAL_JOBS_MOD_RS: &str = r#"use core_jobs::worker::Worker;

#[allow(unused_variables)]
pub fn register_jobs(worker: &mut Worker) {}

#[allow(unused_variables)]
pub fn register_schedules(scheduler: &mut core_jobs::cron::Scheduler) {}
"#;

pub const APP_INTERNAL_DATATABLES_MOD_RS: &str = r#"include!("mod.generated.rs");
"#;

pub const APP_SEEDS_MOD_RS: &str = r#"pub mod admin_bootstrap_seeder;
pub mod countries_seeder;

pub fn register_seeders(seeders: &mut Vec<Box<dyn core_db::seeder::Seeder>>) {
    seeders.push(Box::new(countries_seeder::CountriesSeeder));
    seeders.push(Box::new(admin_bootstrap_seeder::AdminBootstrapSeeder));
}
"#;

pub const APP_SEEDS_COUNTRIES_RS: &str = r#"use async_trait::async_trait;
use core_db::{
    common::sql::DbConn,
    platform::countries::repo::CountryRepo,
    seeder::Seeder,
};

#[derive(Debug, Default)]
pub struct CountriesSeeder;

#[async_trait]
impl Seeder for CountriesSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        CountryRepo::new(DbConn::pool(db)).seed_builtin().await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "CountriesSeeder"
    }
}
"#;

pub const APP_SEEDS_ADMIN_BOOTSTRAP_RS: &str = r#"use async_trait::async_trait;
use core_db::{
    common::{
        auth::hash::hash_password,
        sql::DbConn,
    },
    platform::auth_subject_permissions::repo::AuthSubjectPermissionRepo,
    seeder::Seeder,
};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct AdminBootstrapSeeder;

#[async_trait]
impl Seeder for AdminBootstrapSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        if should_skip_in_env() {
            return Ok(());
        }

        let developer_id = upsert_admin(
            db,
            &env_or("SEED_ADMIN_DEVELOPER_EMAIL", "developer@example.com"),
            &env_or("SEED_ADMIN_DEVELOPER_PASSWORD", "password123"),
            "Developer",
            "developer",
        )
        .await?;

        let superadmin_id = upsert_admin(
            db,
            &env_or("SEED_ADMIN_SUPERADMIN_EMAIL", "superadmin@example.com"),
            &env_or("SEED_ADMIN_SUPERADMIN_PASSWORD", "password123"),
            "Super Admin",
            "superadmin",
        )
        .await?;

        let repo = AuthSubjectPermissionRepo::new(DbConn::pool(db));
        repo.replace("admin", developer_id, &["*".to_string()]).await?;

        let super_permissions = generated::permissions::Permission::all()
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect::<Vec<_>>();
        repo.replace("admin", superadmin_id, &super_permissions).await?;

        Ok(())
    }

    fn name(&self) -> &str {
        "AdminBootstrapSeeder"
    }
}

fn should_skip_in_env() -> bool {
    let app_env = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".to_string())
        .trim()
        .to_ascii_lowercase();

    if app_env != "production" {
        return false;
    }

    !is_truthy(&std::env::var("SEED_ADMIN_BOOTSTRAP_IN_PROD").unwrap_or_default())
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "y"
    )
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

async fn upsert_admin(
    db: &sqlx::PgPool,
    email: &str,
    password_plain: &str,
    name: &str,
    admin_type: &str,
) -> anyhow::Result<Uuid> {
    let password = hash_password(password_plain)?;

    let id = sqlx::query_scalar::<_, Uuid>(
        "\n        INSERT INTO admin (email, password, name, admin_type)\n        VALUES ($1, $2, $3, $4)\n        ON CONFLICT (email) DO UPDATE\n        SET\n            password = EXCLUDED.password,\n            name = EXCLUDED.name,\n            admin_type = EXCLUDED.admin_type,\n            updated_at = NOW()\n        RETURNING id\n        ",
    )
    .bind(email)
    .bind(password)
    .bind(name)
    .bind(admin_type)
    .fetch_one(db)
    .await?;

    Ok(id)
}
"#;

pub const APP_BIN_API_SERVER_RS: &str = r#"#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::web::start_server(
        app::internal::api::build_router,
        |ctx| async move {
            bootstrap::jobs::start_with_context(
                ctx,
                app::internal::jobs::register_jobs,
                Some(app::internal::jobs::register_schedules),
            )
            .await
        },
    )
    .await
}
"#;

pub const APP_BIN_WEBSOCKET_SERVER_RS: &str = r#"#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::realtime::start_server(
        |_ctx| async move { Ok(axum::Router::new()) },
        |_ctx| async move { Ok(()) },
        bootstrap::realtime::RealtimeStartOptions::default(),
    )
    .await
}
"#;

pub const APP_BIN_WORKER_RS: &str = r#"#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::jobs::start_worker(
        app::internal::jobs::register_jobs,
        Some(app::internal::jobs::register_schedules),
    )
    .await
}
"#;

pub const APP_BIN_CONSOLE_RS: &str = r#"use bootstrap::boot::BootContext;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Health check for project command wiring.
    Ping,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, _ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Ping => {
                println!("pong");
            }
        }
        Ok(())
    }
}

fn register_seeders(seeders: &mut Vec<Box<dyn core_db::seeder::Seeder>>) {
    app::seeds::register_seeders(seeders);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::console::start_console::<ProjectCommands, fn(&mut Vec<Box<dyn core_db::seeder::Seeder>>)>(Some(register_seeders))
        .await
}
"#;

pub const GENERATED_CARGO_TOML: &str = r#"[package]
name = "generated"
version = "0.1.0"
edition.workspace = true

[dependencies]
core-db = { workspace = true }
core-datatable = { workspace = true }
core-i18n = { workspace = true }
core-web = { workspace = true }
core-jobs = { workspace = true }
core-notify = { workspace = true }

serde = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
anyhow = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
schemars = { workspace = true }
validator = { workspace = true }
time = { workspace = true }
uuid = { workspace = true }

[build-dependencies]
db-gen = { workspace = true }
toml = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
"#;

pub const GENERATED_BUILD_RS: &str = r##"fn main() {
    let app_dir = std::path::Path::new("..").join("app");
    let configs_path = app_dir.join("configs.toml");
    let permissions_path = app_dir.join("permissions.toml");
    let schemas_dir = app_dir.join("schemas");
    let out_dir = std::path::Path::new("src");

    println!("cargo:rerun-if-changed={}", configs_path.display());
    println!("cargo:rerun-if-changed={}", permissions_path.display());
    println!("cargo:rerun-if-changed={}", schemas_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    let (cfgs, _) =
        db_gen::config::load(configs_path.to_str().unwrap()).expect("Failed to load configs");

    let schema =
        db_gen::schema::load(schemas_dir.to_str().unwrap()).expect("Failed to load schemas");
    let permissions = db_gen::load_permissions(permissions_path.to_str().unwrap())
        .expect("Failed to load permissions");

    let models_out = out_dir.join("models");
    if !models_out.exists() {
        std::fs::create_dir_all(&models_out).expect("Failed to create models out");
    }
    db_gen::generate_enums(&schema, &models_out).expect("Failed to gen enums");
    db_gen::generate_models(&schema, &cfgs, &models_out).expect("Failed to gen models");

    let guards_out = out_dir.join("guards");
    if !guards_out.exists() {
        std::fs::create_dir_all(&guards_out).expect("Failed to create guards out");
    }
    db_gen::generate_auth(&cfgs, &guards_out).expect("Failed to gen auth");
    db_gen::generate_permissions(&permissions, &out_dir.join("permissions.rs"))
        .expect("Failed to gen permissions");

    db_gen::generate_localized(&cfgs.languages, &cfgs, &schema, out_dir)
        .expect("Failed to gen localized");

    let app_datatables_out = app_dir.join("src").join("internal").join("datatables");
    db_gen::generate_datatable_skeletons(&schema, &app_datatables_out)
        .expect("Failed to gen app datatable skeletons");

    let root_lib = out_dir.join("lib.rs");
    let mut f = std::fs::File::create(&root_lib).expect("Failed to create root lib.rs");
    use std::io::Write;
    writeln!(f, "#![allow(dead_code)]").unwrap();
    writeln!(f, "// AUTO-GENERATED FILE — DO NOT EDIT").unwrap();
    writeln!(f, "pub mod models;").unwrap();
    writeln!(f, "pub mod guards;").unwrap();
    writeln!(f, "pub mod permissions;").unwrap();
    writeln!(f, "pub mod localized;").unwrap();
    writeln!(f, "pub use localized::*;").unwrap();
    writeln!(f, "pub mod extensions;").unwrap();
    writeln!(f, "pub mod generated {{ pub use crate::*; }}").unwrap();
}
"##;

pub const GENERATED_LIB_RS: &str = r#"// Placeholder before first generated/build.rs execution.
pub mod extensions;
"#;

pub const GENERATED_EXTENSIONS_RS: &str = r#"// Manual extensions and strongly typed custom model shapes.
// Safe to edit.

pub mod admin {
    pub mod types {}
}
"#;
