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
tower-cookies = "0.11"
ts-rs = { version = "10", features = ["serde-compat"] }

bootstrap = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-config = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-db = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-datatable = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-mailer = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-i18n = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-jobs = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-notify = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-realtime = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
core-web = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
db-gen = { git = "https://github.com/weiloon1234/Rustforge.git", branch = "main" }
"#;

pub const ROOT_ENV_EXAMPLE: &str = r#"# ----------------------------
# App
# ----------------------------
APP_NAME=starter
APP_ENV=local
APP_DEBUG=false
APP_KEY={{APP_KEY}}
APP_TIMEZONE=+08:00
RUST_LOG=info
DEFAULT_PER_PAGE=30
DATATABLE_UNKNOWN_FILTER_MODE=ignore
DATATABLE_EXPORT_LINK_TTL_SECS=604800
# Used by scripts/install-ubuntu.sh to run services under an isolated Linux user.
# Leave empty for local dev; installer will prompt and persist it.
PROJECT_USER=
# Installer writes this using selected project slug for supervisor/nginx naming.
# Leave empty to auto-derive "{APP_NAME}-{APP_ENV}" in scripts/update.sh.
SUPERVISOR_PROJECT_SLUG=

# ----------------------------
# Paths
# ----------------------------
APP_CONFIGS_PATH=app/configs.toml
APP_MIGRATIONS_DIR=migrations
APP_SEEDERS_DIR=app/src/seeds
PUBLIC_PATH=public

# ----------------------------
# i18n
# ----------------------------
I18N_DIR=i18n

# ----------------------------
# Framework docs / OpenAPI
# ----------------------------
ENABLE_FRAMEWORK_DOCS=false
FRAMEWORK_DOCS_PATH=/framework-documentation
# Optional explicit docs assets directory override.
# Default resolution is PUBLIC_PATH + FRAMEWORK_DOCS_PATH.
FRAMEWORK_DOCS_DIST_DIR=
ENABLE_OPENAPI_DOCS=true
OPENAPI_DOCS_PATH=/openapi
OPENAPI_JSON_PATH=/openapi.json

# ----------------------------
# Server
# ----------------------------
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# ----------------------------
# Realtime
# ----------------------------
REALTIME_ENABLED=true
REALTIME_HOST=127.0.0.1
REALTIME_PORT=3010
REALTIME_REQUIRE_AUTH=false
REALTIME_HEARTBEAT_SECS=20
REALTIME_PRESENCE_TTL_SECS=60
REALTIME_MAX_CONNECTIONS=10000
REALTIME_MAX_MESSAGE_BYTES=65536
REALTIME_MAX_FRAME_BYTES=65536
REALTIME_MAX_MESSAGES_PER_SEC=150
REALTIME_SEND_QUEUE_CAPACITY=1024
REALTIME_CHECKPOINT_ENABLED=false
REALTIME_CHECKPOINT_TTL_SECS=2592000
REALTIME_DELIVERY_MODE=at_most_once
REALTIME_STREAM_MAX_LEN=100000
REALTIME_STREAM_RETENTION_SECS=0
REALTIME_REPLAY_LIMIT_DEFAULT=200
REALTIME_REPLAY_LIMIT_MAX=1000
REALTIME_REPLAY_GAP_ALERT_THRESHOLD=100
REALTIME_REPLAY_GAP_ALERT_WINDOW_SECS=60

# ----------------------------
# Database (Postgres)
# ----------------------------
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/starter
DB_MAX_CONNECTIONS=10
DB_CONNECT_TIMEOUT_SECS=5
# Optional; 0..1023 typical for distributed Snowflake IDs.
SNOWFLAKE_NODE_ID=1

# ----------------------------
# Redis
# ----------------------------
# REDIS_URL has priority. If empty, REDIS_HOST/PORT/PASSWORD/DB will be used.
REDIS_URL=redis://127.0.0.1:6379/0
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DB=0
# Leave empty to auto-derive "{APP_NAME}_{APP_ENV}".
REDIS_CACHE_PREFIX=

# ----------------------------
# Object Storage (S3/R2/MinIO)
# ----------------------------
S3_ENDPOINT=
S3_REGION=auto
S3_BUCKET=
S3_ACCESS_KEY=
S3_SECRET_KEY=
S3_FORCE_PATH_STYLE=false
# Public base URL for file access (CDN/CNAME).
S3_URL=

# ----------------------------
# Mailer
# ----------------------------
MAIL_ENABLE=false
MAIL_DRIVER=log
MAIL_HOST=smtp.mailtrap.io
MAIL_PORT=2525
MAIL_USERNAME=
MAIL_PASSWORD=
MAIL_FROM_ADDRESS=hello@example.com

# ----------------------------
# Middleware
# ----------------------------
MW_RATE_LIMIT_PER_SEC=2
MW_RATE_LIMIT_BURST=60
MW_TIMEOUT_SECS=30
MW_BODY_LIMIT_MB=10

# ----------------------------
# HTTP Logging
# ----------------------------
HTTP_LOG_WEBHOOK_ENABLED=false
HTTP_LOG_WEBHOOK_PATHS=/wh/,/webhook/
HTTP_LOG_CLIENT_ENABLED=false
HTTP_LOG_RETENTION_DAYS=7

# ----------------------------
# Worker
# ----------------------------
RUN_WORKER=false
WORKER_CONCURRENCY=10
WORKER_SWEEP_INTERVAL=30

# ----------------------------
# Admin Bootstrap Seeder
# ----------------------------
SEED_ADMIN_BOOTSTRAP_IN_PROD=false
SEED_ADMIN_DEVELOPER_USERNAME=developer
SEED_ADMIN_DEVELOPER_EMAIL=
SEED_ADMIN_DEVELOPER_PASSWORD=password123
SEED_ADMIN_DEVELOPER_NAME=Developer
SEED_ADMIN_SUPERADMIN_USERNAME=superadmin
SEED_ADMIN_SUPERADMIN_EMAIL=
SEED_ADMIN_SUPERADMIN_PASSWORD=password123
SEED_ADMIN_SUPERADMIN_NAME=Super Admin
"#;

pub const ROOT_GITIGNORE: &str = r#"target/
**/target/
.env
.env.local
.env.*.local
.DS_Store
Thumbs.db
*.log
*.tmp
*.pid
logs/
.idea/
.vscode/
node_modules/
**/node_modules/
frontend/dist/

# Keep the directory, ignore generated static files by default.
public/*
!public/.gitkeep
"#;

pub const ROOT_GITATTRIBUTES: &str = r#"* text=auto eol=lf

*.png binary
*.jpg binary
*.jpeg binary
*.gif binary
*.ico binary
*.bmp binary
*.tiff binary
*.pdf binary
*.zip binary
*.tar binary
*.gz binary
*.bz2 binary
*.xz binary
*.7z binary
*.rar binary
*.woff binary
*.woff2 binary
*.ttf binary
*.otf binary
*.mp3 binary
*.mp4 binary
*.mov binary
*.avi binary
*.webm binary
"#;

pub const ROOT_MAKEFILE: &str = r#"SHELL := /bin/bash
RUSTFORGE_PATH ?= ../Rustforge

ifneq (,$(wildcard ./.env))
	include ./.env
	export
endif

PUBLIC_PATH ?= public
FRAMEWORK_DOCS_PATH ?= /framework-documentation
FRAMEWORK_DOCS_ROUTE := $(patsubst /%,%,$(FRAMEWORK_DOCS_PATH))
FRAMEWORK_DOCS_DIR := $(PUBLIC_PATH)/$(FRAMEWORK_DOCS_ROUTE)

.PHONY: help
help:
	@echo "Starter Makefile"
	@echo "--------------"
	@echo "  make dev                 # Rust API + all Vite portals"
	@echo "  make dev-api             # Rust API only (cargo-watch)"
	@echo "  make dev-frontend        # All Vite portals"
	@echo "  make dev-user            # Vite user portal only"
	@echo "  make dev-admin           # Vite admin portal only"
	@echo "  make install-frontend    # npm install for frontend"
	@echo "  make build-frontend      # Production build all portals"
	@echo "  make run-api"
	@echo "  make run-ws"
	@echo "  make run-worker"
	@echo "  make console CMD='route list'"
	@echo "  make route-list"
	@echo "  make migrate-pump"
	@echo "  make migrate-run"
	@echo "  make server-install"
	@echo "  make server-update"
	@echo "  make assets-publish ASSETS_ARGS='--from frontend/dist --clean'"
	@echo "  make framework-docs-build"
	@echo "  make check"
	@echo "  make gen"
	@echo "  make gen-types            # Regenerate frontend TS types from Rust contracts"

.PHONY: install-tools
install-tools:
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch

.PHONY: install-frontend
install-frontend:
	npm --prefix frontend install

.PHONY: dev-api
dev-api:
	@command -v cargo-watch >/dev/null 2>&1 || (echo "cargo-watch not found. Run: make install-tools" && exit 1)
	RUN_WORKER=true cargo watch -x "run -p app --bin api-server"

.PHONY: dev-user
dev-user:
	npm --prefix frontend run dev:user

.PHONY: dev-admin
dev-admin:
	npm --prefix frontend run dev:admin

.PHONY: dev-frontend
dev-frontend:
	@trap 'kill 0' EXIT; \
	npm --prefix frontend run dev:user & \
	npm --prefix frontend run dev:admin & \
	wait

.PHONY: dev
dev:
	@command -v cargo-watch >/dev/null 2>&1 || (echo "cargo-watch not found. Run: make install-tools" && exit 1)
	@trap 'kill 0' EXIT; \
	RUN_WORKER=true cargo watch -x "run -p app --bin api-server" & \
	npm --prefix frontend run dev:user & \
	npm --prefix frontend run dev:admin & \
	wait

.PHONY: build-frontend
build-frontend:
	npm --prefix frontend run build

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
	./console $(CMD)

.PHONY: route-list
route-list:
	./console route list

.PHONY: migrate-pump
migrate-pump:
	./console migrate pump

.PHONY: migrate-run
migrate-run:
	./console migrate run

.PHONY: server-install
server-install:
	sudo ./scripts/install-ubuntu.sh

.PHONY: server-update
server-update:
	./scripts/update.sh

.PHONY: assets-publish
assets-publish:
	./console assets publish $(ASSETS_ARGS)

.PHONY: framework-docs-build
framework-docs-build:
	npm --prefix $(RUSTFORGE_PATH)/core-docs/frontend run build
	@mkdir -p "$(FRAMEWORK_DOCS_DIR)"
	@find "$(FRAMEWORK_DOCS_DIR)" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
	cp -R "$(RUSTFORGE_PATH)/core-docs/frontend/dist/." "$(FRAMEWORK_DOCS_DIR)/"
	@echo "Published framework docs assets to $(FRAMEWORK_DOCS_DIR)"

.PHONY: check
check:
	cargo check --workspace

.PHONY: gen-types
gen-types:
	cargo run -p app --bin export-types
	@echo "TypeScript types regenerated in frontend/src/"

.PHONY: gen
gen:
	cargo build -p generated
	$(MAKE) gen-types
"#;

pub const ROOT_README_MD: &str = r#"# Rustforge Starter

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
"#;

pub const ROOT_I18N_EN_JSON: &str = r#"{
  "Admin list loaded": "Admin list loaded",
  "Admin loaded": "Admin loaded",
  "Admin created": "Admin created",
  "Admin updated": "Admin updated",
  "Admin deleted": "Admin deleted",
  "Username is already taken": "Username is already taken",
  "Cannot assign permissions you do not have": "Cannot assign permissions you do not have",
  "You cannot update your own admin account here": "You cannot update your own admin account here",
  "You cannot delete your own admin account here": "You cannot delete your own admin account here",
  "Normal admin cannot assign admin.read or admin.manage": "Normal admin cannot assign admin.read or admin.manage",
  "Profile loaded": "Profile loaded",
  "Login successful": "Login successful",
  "Token refreshed": "Token refreshed",
  "Logout successful": "Logout successful",
  "Profile updated successfully": "Profile updated successfully",
  "Password updated successfully": "Password updated successfully",
  "Current password is incorrect": "Current password is incorrect",
  "Admin not found": "Admin not found",
  "Missing refresh token": "Missing refresh token",
  "Invalid credentials": "Invalid credentials",
  "Access denied": "Access denied"
}
"#;

pub const ROOT_I18N_ZH_JSON: &str = r#"{
  "Admin list loaded": "管理员列表已加载",
  "Admin loaded": "管理员资料已加载",
  "Admin created": "管理员创建成功",
  "Admin updated": "管理员更新成功",
  "Admin deleted": "管理员删除成功",
  "Username is already taken": "用户名已被使用",
  "Cannot assign permissions you do not have": "不能分配你没有的权限",
  "You cannot update your own admin account here": "不能在这里修改你自己的管理员账号",
  "You cannot delete your own admin account here": "不能在这里删除你自己的管理员账号",
  "Normal admin cannot assign admin.read or admin.manage": "普通管理员不能分配 admin.read 或 admin.manage",
  "Profile loaded": "个人资料已加载",
  "Login successful": "登录成功",
  "Token refreshed": "令牌已刷新",
  "Logout successful": "登出成功",
  "Profile updated successfully": "个人资料更新成功",
  "Password updated successfully": "密码更新成功",
  "Current password is incorrect": "当前密码不正确",
  "Admin not found": "找不到管理员",
  "Missing refresh token": "缺少刷新令牌",
  "Invalid credentials": "凭证无效",
  "Access denied": "拒绝访问"
}
"#;

pub const ROOT_CONSOLE: &str = r#"#!/usr/bin/env bash
set -euo pipefail
./bin/console "$@"
"#;

pub const SCRIPT_INSTALL_UBUNTU_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    echo "Run as root: sudo ./scripts/install-ubuntu.sh"
    exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR_DEFAULT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

prompt() {
    local label="$1"
    local default_value="${2:-}"
    local value
    if [[ -n "${default_value}" ]]; then
        read -r -p "${label} [${default_value}]: " value
        printf "%s" "${value:-$default_value}"
        return
    fi
    read -r -p "${label}: " value
    printf "%s" "${value}"
}

prompt_yes_no() {
    local label="$1"
    local default_value="${2:-yes}"
    local raw
    raw="$(prompt "${label} (yes/no)" "${default_value}")"
    raw="$(printf "%s" "$raw" | tr '[:upper:]' '[:lower:]')"
    case "${raw}" in
        y | yes | true | 1) printf "yes" ;;
        n | no | false | 0) printf "no" ;;
        *)
            echo "Invalid input: ${raw}. Expected yes or no." >&2
            exit 1
            ;;
    esac
}

slugify() {
    printf "%s" "$1" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g; s/-\{2,\}/-/g; s/^-//; s/-$//'
}

normalize_username() {
    local value
    value="$(printf "%s" "$1" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9_-]/-/g; s/^-*//; s/-*$//')"
    if [[ -z "${value}" ]]; then
        value="appuser"
    fi
    if [[ "${value}" =~ ^[0-9] ]]; then
        value="u${value}"
    fi
    printf "%s" "${value}"
}

read_env_value() {
    local file="$1"
    local key="$2"
    if [[ -f "${file}" ]]; then
        grep -E "^${key}=" "${file}" | head -n1 | sed "s/^${key}=//" || true
    fi
}

upsert_env() {
    local file="$1"
    local key="$2"
    local value="$3"
    local escaped
    escaped="$(printf '%s' "${value}" | sed -e 's/[\/&]/\\&/g')"
    if grep -qE "^${key}=" "${file}"; then
        sed -i "s/^${key}=.*/${key}=${escaped}/" "${file}"
    else
        printf "%s=%s\n" "${key}" "${value}" >> "${file}"
    fi
}

write_file_if_changed() {
    local target="$1"
    local mode="$2"
    local content="$3"
    local tmp
    tmp="$(mktemp)"
    printf "%s" "${content}" > "${tmp}"

    if [[ -f "${target}" ]] && cmp -s "${tmp}" "${target}"; then
        rm -f "${tmp}"
        return 1
    fi

    if [[ -f "${target}" ]]; then
        cp "${target}" "${target}.bak.$(date +%Y%m%d%H%M%S)"
    fi

    install -m "${mode}" "${tmp}" "${target}"
    rm -f "${tmp}"
    return 0
}

ensure_packages() {
    local missing=()
    local pkg
    for pkg in "$@"; do
        if ! dpkg -s "${pkg}" >/dev/null 2>&1; then
            missing+=("${pkg}")
        fi
    done
    if (( ${#missing[@]} > 0 )); then
        apt-get update -y
        apt-get install -y "${missing[@]}"
    fi
}

run_as_user() {
    local user="$1"
    local command="$2"
    if command -v sudo >/dev/null 2>&1; then
        sudo -u "${user}" -H env PROJECT_DIR="${PROJECT_DIR}" bash -lc "${command}"
    else
        su - "${user}" -c "PROJECT_DIR='${PROJECT_DIR}' bash -lc '${command}'"
    fi
}

ensure_root_cron_entry() {
    local tag="$1"
    local line="$2"
    local existing
    existing="$(crontab -l 2>/dev/null || true)"
    if grep -Fq "${tag}" <<<"${existing}"; then
        return
    fi
    {
        printf "%s\n" "${existing}"
        printf "%s # %s\n" "${line}" "${tag}"
    } | awk 'NF' | crontab -
}

bool_value() {
    if [[ "$1" == "yes" ]]; then
        printf "true"
    else
        printf "false"
    fi
}

append_ssh_key_if_missing() {
    local file="$1"
    local key="$2"
    [[ -z "${key}" ]] && return
    touch "${file}"
    if ! grep -Fxq "${key}" "${file}"; then
        printf "%s\n" "${key}" >> "${file}"
    fi
}

if [[ ! -f /etc/os-release ]]; then
    echo "Cannot detect OS. /etc/os-release is missing."
    exit 1
fi
source /etc/os-release
if [[ "${ID:-}" != "ubuntu" ]]; then
    echo "This installer supports Ubuntu only."
    exit 1
fi

ubuntu_major="${VERSION_ID%%.*}"
if [[ "${ubuntu_major}" != "24" && "${ubuntu_major}" != "25" ]]; then
    echo "Detected Ubuntu ${VERSION_ID}. Supported targets are Ubuntu 24 or 25."
    if [[ "$(prompt_yes_no "Continue anyway?" "no")" != "yes" ]]; then
        exit 1
    fi
fi

echo "Rustforge Starter Server Installer (idempotent)"
echo "It is safe to run this script multiple times for the same project."
echo

PROJECT_DIR="$(prompt "Project directory" "${PROJECT_DIR_DEFAULT}")"
if [[ ! -d "${PROJECT_DIR}" ]]; then
    echo "Project directory does not exist: ${PROJECT_DIR}"
    exit 1
fi
if [[ ! -f "${PROJECT_DIR}/Cargo.toml" ]]; then
    echo "Cargo.toml not found in ${PROJECT_DIR}."
    exit 1
fi
if [[ ! -f "${PROJECT_DIR}/bin/api-server" ]]; then
    echo "Expected starter bin scripts under ${PROJECT_DIR}/bin."
    exit 1
fi

ENV_FILE="${PROJECT_DIR}/.env"
if [[ ! -f "${ENV_FILE}" ]]; then
    if [[ -f "${PROJECT_DIR}/.env.example" ]]; then
        cp "${PROJECT_DIR}/.env.example" "${ENV_FILE}"
    else
        touch "${ENV_FILE}"
    fi
fi

existing_app_name="$(read_env_value "${ENV_FILE}" "APP_NAME")"
APP_NAME="$(prompt "APP_NAME" "${existing_app_name:-$(basename "${PROJECT_DIR}")}")"
PROJECT_SLUG="$(prompt "Project slug (used for nginx/supervisor file names)" "$(slugify "${APP_NAME}")")"
DOMAIN="$(prompt "Domain (example: api.example.com)" "example.com")"

existing_project_user="$(read_env_value "${ENV_FILE}" "PROJECT_USER")"
default_project_user="$(normalize_username "${existing_project_user:-$PROJECT_SLUG}")"
PROJECT_USER="$(normalize_username "$(prompt "Isolated Linux user for this project" "${default_project_user}")")"

SSH_AUTH_MODE="$(prompt "SSH auth for isolated user (copy-root-key/manual-key/generate-password)" "copy-root-key")"
SSH_AUTH_MODE="$(printf "%s" "${SSH_AUTH_MODE}" | tr '[:upper:]' '[:lower:]')"
case "${SSH_AUTH_MODE}" in
    copy-root-key | manual-key | generate-password) ;;
    *)
        echo "Invalid SSH auth mode: ${SSH_AUTH_MODE}"
        exit 1
        ;;
esac
MANUAL_SSH_KEY=""
if [[ "${SSH_AUTH_MODE}" == "manual-key" ]]; then
    MANUAL_SSH_KEY="$(prompt "Paste public SSH key for ${PROJECT_USER}")"
    if [[ -z "${MANUAL_SSH_KEY}" ]]; then
        echo "Public SSH key is required for manual-key mode."
        exit 1
    fi
fi

existing_env="$(read_env_value "${ENV_FILE}" "APP_ENV")"
APP_ENV="$(prompt "APP_ENV" "${existing_env:-production}")"
debug_default="no"
if [[ "$(read_env_value "${ENV_FILE}" "APP_DEBUG")" == "true" ]]; then
    debug_default="yes"
fi
APP_DEBUG="$(prompt_yes_no "APP_DEBUG" "${debug_default}")"

server_port_default="$(read_env_value "${ENV_FILE}" "SERVER_PORT")"
realtime_port_default="$(read_env_value "${ENV_FILE}" "REALTIME_PORT")"
SERVER_PORT="$(prompt "SERVER_PORT" "${server_port_default:-3000}")"
REALTIME_PORT="$(prompt "REALTIME_PORT" "${realtime_port_default:-3010}")"

db_default="$(read_env_value "${ENV_FILE}" "DATABASE_URL")"
redis_default="$(read_env_value "${ENV_FILE}" "REDIS_URL")"
DATABASE_URL="$(prompt "DATABASE_URL" "${db_default:-postgres://postgres:postgres@127.0.0.1:5432/${PROJECT_SLUG}}")"
REDIS_URL="$(prompt "REDIS_URL" "${redis_default:-redis://127.0.0.1:6379/0}")"

ENABLE_HTTPS="$(prompt_yes_no "Enable HTTPS with Let's Encrypt" "yes")"
LETSENCRYPT_EMAIL=""
if [[ "${ENABLE_HTTPS}" == "yes" ]]; then
    LETSENCRYPT_EMAIL="$(prompt "Let's Encrypt email" "admin@${DOMAIN}")"
fi

ENABLE_SUPERVISOR="$(prompt_yes_no "Enable Supervisor process management" "yes")"
ENABLE_WS="$(prompt_yes_no "Manage websocket-server process" "yes")"
ENABLE_WORKER="$(prompt_yes_no "Manage worker process" "yes")"

BUILD_RELEASE="$(prompt_yes_no "Build release binaries now" "yes")"
RUN_MIGRATIONS="$(prompt_yes_no "Run ./console migrate run now" "yes")"

echo
echo "Summary:"
echo "  Project dir      : ${PROJECT_DIR}"
echo "  Project user     : ${PROJECT_USER}"
echo "  SSH auth mode    : ${SSH_AUTH_MODE}"
echo "  Domain           : ${DOMAIN}"
echo "  APP_ENV          : ${APP_ENV}"
echo "  Supervisor slug  : ${PROJECT_SLUG}"
echo "  HTTPS            : ${ENABLE_HTTPS}"
echo "  Supervisor       : ${ENABLE_SUPERVISOR}"
echo "  Websocket proc   : ${ENABLE_WS}"
echo "  Worker proc      : ${ENABLE_WORKER}"
echo
if [[ "$(prompt_yes_no "Proceed with installation?" "yes")" != "yes" ]]; then
    echo "Cancelled."
    exit 0
fi

USER_CREATED="no"
GENERATED_PASSWORD=""
if ! id -u "${PROJECT_USER}" >/dev/null 2>&1; then
    useradd -m -s /bin/bash "${PROJECT_USER}"
    USER_CREATED="yes"
    echo "Created isolated user: ${PROJECT_USER}"
fi

project_home="$(getent passwd "${PROJECT_USER}" | cut -d: -f6)"
if [[ -z "${project_home}" ]]; then
    echo "Failed to resolve home directory for ${PROJECT_USER}."
    exit 1
fi

mkdir -p "${project_home}/.ssh"
touch "${project_home}/.ssh/authorized_keys"
chmod 700 "${project_home}/.ssh"
chmod 600 "${project_home}/.ssh/authorized_keys"

if [[ "${SSH_AUTH_MODE}" == "copy-root-key" ]]; then
    if [[ -f /root/.ssh/authorized_keys ]]; then
        while IFS= read -r line; do
            append_ssh_key_if_missing "${project_home}/.ssh/authorized_keys" "${line}"
        done </root/.ssh/authorized_keys
    else
        echo "Warning: /root/.ssh/authorized_keys not found. No key copied."
    fi
elif [[ "${SSH_AUTH_MODE}" == "manual-key" ]]; then
    append_ssh_key_if_missing "${project_home}/.ssh/authorized_keys" "${MANUAL_SSH_KEY}"
fi

if [[ "${SSH_AUTH_MODE}" == "generate-password" ]]; then
    if [[ "${USER_CREATED}" == "yes" || "$(prompt_yes_no "User exists. Rotate password for ${PROJECT_USER}?" "no")" == "yes" ]]; then
        ensure_packages openssl
        GENERATED_PASSWORD="$(openssl rand -base64 18 | tr -d '=+/' | cut -c1-20)"
        echo "${PROJECT_USER}:${GENERATED_PASSWORD}" | chpasswd
    fi
else
    passwd -l "${PROJECT_USER}" >/dev/null 2>&1 || true
fi

chown -R "${PROJECT_USER}:${PROJECT_USER}" "${project_home}/.ssh"
chown -R "${PROJECT_USER}:${PROJECT_USER}" "${PROJECT_DIR}"

if ! command -v nginx >/dev/null 2>&1; then
    if [[ "$(prompt_yes_no "nginx is not installed. Install nginx now?" "yes")" != "yes" ]]; then
        echo "nginx is required."
        exit 1
    fi
    ensure_packages nginx
fi

if [[ "${ENABLE_SUPERVISOR}" == "yes" ]]; then
    ensure_packages supervisor
fi

if [[ "${ENABLE_HTTPS}" == "yes" ]]; then
    ensure_packages certbot python3-certbot-nginx cron
fi

if [[ "${BUILD_RELEASE}" == "yes" ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
        if [[ "$(prompt_yes_no "cargo is missing. Install Rust toolchain for ${PROJECT_USER}?" "yes")" != "yes" ]]; then
            echo "cargo is required to build binaries."
            exit 1
        fi
        ensure_packages curl ca-certificates build-essential pkg-config libssl-dev
        run_as_user "${PROJECT_USER}" "curl https://sh.rustup.rs -sSf | sh -s -- -y"
    fi
    run_as_user "${PROJECT_USER}" "source \"\$HOME/.cargo/env\" >/dev/null 2>&1 || true; cd \"\$PROJECT_DIR\" && cargo build --release --workspace"
fi

upsert_env "${ENV_FILE}" "APP_NAME" "${APP_NAME}"
upsert_env "${ENV_FILE}" "APP_ENV" "${APP_ENV}"
upsert_env "${ENV_FILE}" "APP_DEBUG" "$(bool_value "${APP_DEBUG}")"
upsert_env "${ENV_FILE}" "PROJECT_USER" "${PROJECT_USER}"
upsert_env "${ENV_FILE}" "SUPERVISOR_PROJECT_SLUG" "${PROJECT_SLUG}"
upsert_env "${ENV_FILE}" "SERVER_HOST" "127.0.0.1"
upsert_env "${ENV_FILE}" "SERVER_PORT" "${SERVER_PORT}"
upsert_env "${ENV_FILE}" "REALTIME_HOST" "127.0.0.1"
upsert_env "${ENV_FILE}" "REALTIME_PORT" "${REALTIME_PORT}"
upsert_env "${ENV_FILE}" "REALTIME_ENABLED" "$(bool_value "${ENABLE_WS}")"
upsert_env "${ENV_FILE}" "DATABASE_URL" "${DATABASE_URL}"
upsert_env "${ENV_FILE}" "REDIS_URL" "${REDIS_URL}"
upsert_env "${ENV_FILE}" "RUN_WORKER" "$(bool_value "${ENABLE_WORKER}")"

if [[ "${RUN_MIGRATIONS}" == "yes" ]]; then
    run_as_user "${PROJECT_USER}" "cd \"\$PROJECT_DIR\" && ./console migrate run"
fi

NGINX_CONF_PATH="/etc/nginx/sites-available/${PROJECT_SLUG}.conf"
NGINX_LINK_PATH="/etc/nginx/sites-enabled/${PROJECT_SLUG}.conf"

NGINX_CONF_CONTENT="$(cat <<EOF
server {
    listen 80;
    listen [::]:80;
    server_name ${DOMAIN};

    client_max_body_size 20m;

    location /ws/ {
        proxy_pass http://127.0.0.1:${REALTIME_PORT}/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location / {
        proxy_pass http://127.0.0.1:${SERVER_PORT};
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF
)"

write_file_if_changed "${NGINX_CONF_PATH}" "0644" "${NGINX_CONF_CONTENT}" || true
ln -sfn "${NGINX_CONF_PATH}" "${NGINX_LINK_PATH}"
nginx -t
systemctl enable --now nginx
systemctl reload nginx

if [[ "${ENABLE_HTTPS}" == "yes" ]]; then
    certbot --nginx -d "${DOMAIN}" --agree-tos --non-interactive --email "${LETSENCRYPT_EMAIL}" --redirect --keep-until-expiring
    ensure_root_cron_entry "rustforge-certbot-${PROJECT_SLUG}" "17 3 * * * certbot renew --quiet --deploy-hook \"systemctl reload nginx\""
fi

if [[ "${ENABLE_SUPERVISOR}" == "yes" ]]; then
    SUPERVISOR_CONF_PATH="/etc/supervisor/conf.d/${PROJECT_SLUG}.conf"
    api_command="./bin/api-server"
    ws_command="./bin/websocket-server"
    worker_command="./bin/worker"
    if [[ -x "${PROJECT_DIR}/target/release/api-server" ]]; then
        api_command="./target/release/api-server"
    fi
    if [[ -x "${PROJECT_DIR}/target/release/websocket-server" ]]; then
        ws_command="./target/release/websocket-server"
    fi
    if [[ -x "${PROJECT_DIR}/target/release/worker" ]]; then
        worker_command="./target/release/worker"
    fi

    supervisor_content="$(cat <<EOF
[program:${PROJECT_SLUG}-api]
directory=${PROJECT_DIR}
command=${api_command}
autostart=true
autorestart=true
startsecs=5
user=${PROJECT_USER}
stopsignal=TERM
stopasgroup=true
killasgroup=true
stdout_logfile=/var/log/${PROJECT_SLUG}-api.log
stderr_logfile=/var/log/${PROJECT_SLUG}-api.err.log

EOF
)"

    if [[ "${ENABLE_WS}" == "yes" ]]; then
        supervisor_content+=$(cat <<EOF
[program:${PROJECT_SLUG}-ws]
directory=${PROJECT_DIR}
command=${ws_command}
autostart=true
autorestart=true
startsecs=5
user=${PROJECT_USER}
stopsignal=TERM
stopasgroup=true
killasgroup=true
stdout_logfile=/var/log/${PROJECT_SLUG}-ws.log
stderr_logfile=/var/log/${PROJECT_SLUG}-ws.err.log

EOF
)
    fi

    if [[ "${ENABLE_WORKER}" == "yes" ]]; then
        supervisor_content+=$(cat <<EOF
[program:${PROJECT_SLUG}-worker]
directory=${PROJECT_DIR}
command=${worker_command}
autostart=true
autorestart=true
startsecs=5
user=${PROJECT_USER}
stopsignal=TERM
stopasgroup=true
killasgroup=true
stdout_logfile=/var/log/${PROJECT_SLUG}-worker.log
stderr_logfile=/var/log/${PROJECT_SLUG}-worker.err.log

EOF
)
    fi

    write_file_if_changed "${SUPERVISOR_CONF_PATH}" "0644" "${supervisor_content}" || true
    systemctl enable --now supervisor
    supervisorctl reread
    supervisorctl update
    supervisorctl restart "${PROJECT_SLUG}-api" || supervisorctl start "${PROJECT_SLUG}-api"
    if [[ "${ENABLE_WS}" == "yes" ]]; then
        supervisorctl restart "${PROJECT_SLUG}-ws" || supervisorctl start "${PROJECT_SLUG}-ws"
    fi
    if [[ "${ENABLE_WORKER}" == "yes" ]]; then
        supervisorctl restart "${PROJECT_SLUG}-worker" || supervisorctl start "${PROJECT_SLUG}-worker"
    fi
fi

echo
echo "Done."
echo "Nginx site : ${NGINX_CONF_PATH}"
echo "Env file   : ${ENV_FILE}"
if [[ "${ENABLE_SUPERVISOR}" == "yes" ]]; then
    echo "Supervisor : /etc/supervisor/conf.d/${PROJECT_SLUG}.conf"
fi
if [[ -n "${GENERATED_PASSWORD}" ]]; then
    echo "SSH login  : ${PROJECT_USER}"
    echo "Password   : ${GENERATED_PASSWORD}"
fi
echo "Try: https://${DOMAIN} (or http://${DOMAIN} when HTTPS is disabled)"
"#;

pub const SCRIPT_UPDATE_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." >/dev/null 2>&1 && pwd)"
ENV_FILE="${PROJECT_DIR}/.env"

read_env_value() {
    local file="$1"
    local key="$2"
    [[ -f "${file}" ]] || return 0
    awk -F= -v k="${key}" '
        $1 ~ "^[[:space:]]*"k"[[:space:]]*$" {
            sub(/^[[:space:]]+/, "", $2)
            sub(/[[:space:]]+$/, "", $2)
            print $2
            exit
        }
    ' "${file}"
}

slugify() {
    printf "%s" "$1" \
        | tr '[:upper:]' '[:lower:]' \
        | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//'
}

run_as_project_user() {
    local command="$1"
    if [[ -n "${PROJECT_USER:-}" && "$(id -u)" -eq 0 ]]; then
        if command -v runuser >/dev/null 2>&1; then
            runuser -u "${PROJECT_USER}" -- bash -lc "${command}"
        elif command -v sudo >/dev/null 2>&1; then
            sudo -u "${PROJECT_USER}" -H bash -lc "${command}"
        else
            su - "${PROJECT_USER}" -c "bash -lc '${command}'"
        fi
    else
        bash -lc "${command}"
    fi
}

run_supervisorctl() {
    if [[ "$(id -u)" -eq 0 ]]; then
        supervisorctl "$@"
        return $?
    fi
    if supervisorctl "$@"; then
        return 0
    fi
    if command -v sudo >/dev/null 2>&1; then
        sudo supervisorctl "$@"
        return $?
    fi
    return 1
}

if [[ ! -d "${PROJECT_DIR}" || ! -f "${PROJECT_DIR}/Cargo.toml" ]]; then
    echo "Invalid project directory: ${PROJECT_DIR}"
    exit 1
fi

APP_NAME="$(read_env_value "${ENV_FILE}" "APP_NAME")"
APP_ENV="$(read_env_value "${ENV_FILE}" "APP_ENV")"
PROJECT_USER="$(read_env_value "${ENV_FILE}" "PROJECT_USER")"
SUPERVISOR_PROJECT_SLUG="$(read_env_value "${ENV_FILE}" "SUPERVISOR_PROJECT_SLUG")"

APP_NAME="${APP_NAME:-$(basename "${PROJECT_DIR}")}"
APP_ENV="${APP_ENV:-production}"
RUN_MIGRATIONS="${RUN_MIGRATIONS:-true}"

if [[ -z "${SUPERVISOR_PROJECT_SLUG}" ]]; then
    candidate_env="$(slugify "${APP_NAME}-${APP_ENV}")"
    candidate_app="$(slugify "${APP_NAME}")"
    if [[ -f "/etc/supervisor/conf.d/${candidate_env}.conf" ]]; then
        SUPERVISOR_PROJECT_SLUG="${candidate_env}"
    elif [[ -f "/etc/supervisor/conf.d/${candidate_app}.conf" ]]; then
        SUPERVISOR_PROJECT_SLUG="${candidate_app}"
    else
        SUPERVISOR_PROJECT_SLUG="${candidate_env}"
    fi
fi

echo "Rustforge Starter Update"
echo "  Project dir      : ${PROJECT_DIR}"
echo "  APP_NAME         : ${APP_NAME}"
echo "  APP_ENV          : ${APP_ENV}"
echo "  Project user     : ${PROJECT_USER:-<current user>}"
echo "  Supervisor slug  : ${SUPERVISOR_PROJECT_SLUG}"
echo "  Run migrations   : ${RUN_MIGRATIONS}"
echo

if [[ -d "${PROJECT_DIR}/.git" ]]; then
    run_as_project_user "cd \"${PROJECT_DIR}\" && git pull --ff-only"
else
    echo "No git repository detected. Skip git pull."
fi

run_as_project_user "source \"\$HOME/.cargo/env\" >/dev/null 2>&1 || true; cd \"${PROJECT_DIR}\" && cargo build --release --workspace"

if [[ -f "${PROJECT_DIR}/frontend/package.json" ]]; then
    run_as_project_user "cd \"${PROJECT_DIR}\" && npm --prefix frontend install && npm --prefix frontend run build"
fi

if [[ "${RUN_MIGRATIONS}" == "true" ]]; then
    run_as_project_user "cd \"${PROJECT_DIR}\" && ./console migrate run"
fi

if command -v supervisorctl >/dev/null 2>&1; then
    SUPERVISOR_CONF_PATH="/etc/supervisor/conf.d/${SUPERVISOR_PROJECT_SLUG}.conf"
    if [[ -f "${SUPERVISOR_CONF_PATH}" ]]; then
        run_supervisorctl reread || true
        run_supervisorctl update || true

        mapfile -t programs < <(grep -oE '^\[program:[^]]+\]' "${SUPERVISOR_CONF_PATH}" | sed -E 's/^\[program:([^]]+)\]$/\1/')
        for program in "${programs[@]}"; do
            [[ -z "${program}" ]] && continue
            run_supervisorctl restart "${program}" || run_supervisorctl start "${program}" || true
        done
    else
        echo "Supervisor config not found at ${SUPERVISOR_CONF_PATH}. Skip restart."
    fi
else
    echo "supervisorctl not found. Skip supervisor restart."
fi

echo "Update completed."
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

# ── CORS ──────────────────────────────────────────────────
# Mirrors Laravel config/cors.php conventions.
# Use ["*"] for development; set explicit origins for production.
[cors]
allowed_origins = ["*"]
allowed_methods = ["*"]
allowed_headers = ["*"]
exposed_headers = []
max_age = 0
supports_credentials = false
"#;

pub const APP_PERMISSIONS_TOML: &str = r#"# Permission catalog (single source of truth).
[[permissions]]
key = "admin.read"
guard = "admin"
label = "Read Admins"
group = "admin"
description = "View admin profile and datatable records."

[[permissions]]
key = "admin.manage"
guard = "admin"
label = "Manage Admins"
group = "admin"
description = "Create/update/delete admin records and perform management actions."
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
pk_type = "i64"
id_strategy = "snowflake"
soft_delete = true
fields = [
  "id:i64",
  "username:string",
  "email:Option<String>",
  "password:hashed",
  "name:string",
  "admin_type:AdminType",
  "abilities:serde_json::Value",
  "created_at:datetime",
  "updated_at:datetime"
]
"#;

pub const MIGRATION_ADMIN_AUTH_SQL: &str = r#"CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS admin (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    username TEXT NOT NULL UNIQUE,
    email TEXT,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    admin_type TEXT NOT NULL CHECK (admin_type IN ('developer', 'superadmin', 'admin')),
    abilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (username = lower(username))
);

CREATE INDEX IF NOT EXISTS idx_admin_username ON admin (username);
CREATE INDEX IF NOT EXISTS idx_admin_admin_type ON admin (admin_type);
CREATE INDEX IF NOT EXISTS idx_admin_email ON admin (email);
"#;

pub const APP_CARGO_TOML: &str = r#"[package]
name = "app"
version = "0.1.0"
edition.workspace = true

[[bin]]
name = "export-types"
path = "src/bin/export-types.rs"

[dependencies]
bootstrap = { workspace = true }
core-config = { workspace = true }
core-db = { workspace = true }
core-datatable = { workspace = true }
core-mailer = { workspace = true }
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
time = { workspace = true }
tower-cookies = { workspace = true }
ts-rs = { workspace = true }
"#;

pub const APP_LIB_RS: &str = r#"pub mod contracts;
pub mod internal;
pub mod seeds;
pub mod validation;
"#;

pub const APP_CONTRACTS_MOD_RS: &str = r#"pub mod api;
pub mod datatable;
pub mod types;
"#;

pub const APP_CONTRACTS_TYPES_MOD_RS: &str = r#"pub mod username;
"#;

pub const APP_CONTRACTS_TYPES_USERNAME_RS: &str = r#"use core_web::contracts::rustforge_string_rule_type;

rustforge_string_rule_type! {
    /// Lowercase username used for admin authentication and admin CRUD inputs.
    pub struct UsernameString {
        #[validate(custom(function = "crate::validation::username::validate_username"))]
        #[rf(length(min = 3, max = 64))]
        #[rf(alpha_dash)]
        #[rf(openapi(description = "Lowercase username using letters, numbers, underscore (_), and hyphen (-).", example = "admin_user"))]
    }
}
"#;

pub const APP_CONTRACTS_API_MOD_RS: &str = r#"pub mod v1;
"#;

pub const APP_CONTRACTS_API_V1_MOD_RS: &str = r#"pub mod admin;
pub mod admin_auth;
"#;

pub const APP_CONTRACTS_DATATABLE_MOD_RS: &str = r#"pub mod admin;
"#;

pub const APP_CONTRACTS_DATATABLE_ADMIN_MOD_RS: &str = r#"pub mod admin;
"#;

pub const APP_CONTRACTS_DATATABLE_ADMIN_ADMIN_RS: &str = r#"use std::collections::BTreeMap;

use core_datatable::DataTableInput;
use core_web::datatable::{
    DataTableEmailExportRequestBase, DataTableFilterFieldDto, DataTableFilterFieldType,
    DataTableQueryRequestBase, DataTableQueryRequestContract, DataTableScopedContract,
};
use core_web::contracts::rustforge_contract;
use generated::models::{AdminType, AdminView};
use schemars::JsonSchema;
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableQueryInput {
    #[serde(default)]
    #[rf(nested)]
    #[ts(type = "DataTableQueryRequestBase")]
    pub base: DataTableQueryRequestBase,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub q: Option<String>,
    #[serde(default)]
    #[rf(length(min = 3, max = 64))]
    #[rf(alpha_dash)]
    pub username: Option<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub email: Option<String>,
    #[serde(default)]
    #[ts(type = "AdminType | null")]
    pub admin_type: Option<AdminType>,
}

impl AdminDatatableQueryInput {
    pub fn to_input(&self) -> DataTableInput {
        let mut input = self.base.to_input();
        let mut params = BTreeMap::new();

        if let Some(q) = self.q.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            params.insert("q".to_string(), q.to_string());
        }
        if let Some(username) = self
            .username
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            params.insert(
                "f-like-username".to_string(),
                username.to_ascii_lowercase(),
            );
        }
        if let Some(email) = self
            .email
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            params.insert("f-like-email".to_string(), email.to_string());
        }
        if let Some(admin_type) = self.admin_type {
            params.insert("f-admin_type".to_string(), admin_type.as_str().to_string());
        }

        input.params.extend(params);
        input
    }
}

impl DataTableQueryRequestContract for AdminDatatableQueryInput {
    fn query_base(&self) -> &DataTableQueryRequestBase {
        &self.base
    }

    fn datatable_query_to_input(&self) -> DataTableInput {
        self.to_input()
    }
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDatatableEmailExportInput {
    #[rf(nested)]
    #[ts(type = "DataTableEmailExportRequestBase")]
    pub base: DataTableEmailExportRequestBase,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub q: Option<String>,
    #[serde(default)]
    #[rf(length(min = 3, max = 64))]
    #[rf(alpha_dash)]
    pub username: Option<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub email: Option<String>,
    #[serde(default)]
    #[ts(type = "AdminType | null")]
    pub admin_type: Option<AdminType>,
}

impl AdminDatatableEmailExportInput {
    pub fn to_input(&self) -> DataTableInput {
        let mut input = self.base.query.to_input();
        let mut params = BTreeMap::new();

        if let Some(q) = self.q.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            params.insert("q".to_string(), q.to_string());
        }
        if let Some(username) = self
            .username
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            params.insert(
                "f-like-username".to_string(),
                username.to_ascii_lowercase(),
            );
        }
        if let Some(email) = self
            .email
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            params.insert("f-like-email".to_string(), email.to_string());
        }
        if let Some(admin_type) = self.admin_type {
            params.insert("f-admin_type".to_string(), admin_type.as_str().to_string());
        }

        input.params.extend(params);
        input.export_file_name = self.base.export_file_name.clone();
        input
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdminAdminDataTableContract;

impl DataTableScopedContract for AdminAdminDataTableContract {
    type QueryRequest = AdminDatatableQueryInput;
    type EmailRequest = AdminDatatableEmailExportInput;
    type Row = AdminView;

    fn scoped_key(&self) -> &'static str {
        "admin.account"
    }

    fn openapi_tag(&self) -> &'static str {
        "Admin Account"
    }

    fn email_to_input(&self, req: &Self::EmailRequest) -> DataTableInput {
        req.to_input()
    }

    fn email_recipients(&self, req: &Self::EmailRequest) -> Vec<String> {
        req.base.recipients.clone()
    }

    fn email_subject(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.subject.clone()
    }

    fn export_file_name(&self, req: &Self::EmailRequest) -> Option<String> {
        req.base.export_file_name.clone()
    }

    fn filter_rows(&self) -> Vec<Vec<DataTableFilterFieldDto>> {
        vec![
            vec![
                DataTableFilterFieldDto {
                    field: "q".to_string(),
                    filter_key: "q".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Keyword".to_string(),
                    placeholder: Some("Search name/username/email".to_string()),
                    description: None,
                    options: None,
                },
                DataTableFilterFieldDto {
                    field: "email".to_string(),
                    filter_key: "f-like-email".to_string(),
                    field_type: DataTableFilterFieldType::Text,
                    label: "Email".to_string(),
                    placeholder: Some("Contains".to_string()),
                    description: None,
                    options: None,
                },
            ],
            vec![DataTableFilterFieldDto {
                field: "username".to_string(),
                filter_key: "f-like-username".to_string(),
                field_type: DataTableFilterFieldType::Text,
                label: "Username".to_string(),
                placeholder: Some("Contains".to_string()),
                description: None,
                options: None,
            }],
            vec![DataTableFilterFieldDto {
                field: "admin_type".to_string(),
                filter_key: "f-admin_type".to_string(),
                field_type: DataTableFilterFieldType::Select,
                label: "Admin Type".to_string(),
                placeholder: Some("Choose type".to_string()),
                description: None,
                options: Some(AdminType::datatable_filter_options()),
            }],
        ]
    }
}
"#;

pub const APP_CONTRACTS_API_V1_ADMIN_RS: &str = r#"use crate::contracts::types::username::UsernameString;
use core_web::contracts::rustforge_contract;
use generated::{models::AdminType, permissions::Permission};
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;
use validator::Validate;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CreateAdminInput {
    #[rf(nested)]
    #[rf(async_unique(table = "admin", column = "username"))]
    #[ts(type = "string")]
    pub username: UsernameString,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[rf(length(min = 1, max = 120))]
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[rf(length(min = 8, max = 128))]
    pub password: String,
    #[serde(default)]
    #[ts(type = "Permission[]")]
    pub abilities: Vec<Permission>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct UpdateAdminInput {
    #[serde(skip, default)]
    __target_id: i64,
    #[serde(default)]
    #[rf(nested)]
    #[rf(async_unique(
        table = "admin",
        column = "username",
        ignore(column = "id", field = "__target_id")
    ))]
    #[ts(type = "string | null")]
    pub username: Option<UsernameString>,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
    #[serde(default)]
    #[rf(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(type = "AdminType | null")]
    pub admin_type: Option<AdminType>,
    #[serde(default)]
    #[ts(type = "Permission[] | null")]
    pub abilities: Option<Vec<Permission>>,
}

impl UpdateAdminInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub updated_at: time::OffsetDateTime,
}

impl From<generated::models::AdminView> for AdminOutput {
    fn from(value: generated::models::AdminView) -> Self {
        let abilities = value
            .abilities
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            name: value.name,
            admin_type: value.admin_type,
            abilities,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminDeleteOutput {
    pub deleted: bool,
}
"#;

pub const APP_CONTRACTS_API_V1_ADMIN_AUTH_RS: &str = r#"use crate::contracts::types::username::UsernameString;
use core_web::contracts::rustforge_contract;
use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;
use validator::Validate;
use core_web::auth::AuthClientType;
use generated::models::AdminType;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLoginInput {
    #[rf(nested)]
    #[ts(type = "string")]
    pub username: UsernameString,

    #[rf(length(min = 8, max = 128))]
    pub password: String,

    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminRefreshInput {
    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLogoutInput {
    #[ts(type = "AuthClientType")]
    pub client_type: AuthClientType,
    #[serde(default)]
    #[rf(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminProfileUpdateInput {
    #[rf(length(min = 1, max = 120))]
    pub name: String,
    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,
}

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    pub current_password: String,
    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,
    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminAuthOutput {
    pub token_type: String,
    pub access_token: String,
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub access_expires_at: Option<time::OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminMeOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminProfileUpdateOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    #[ts(type = "AdminType")]
    pub admin_type: AdminType,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateOutput {
    pub updated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminLogoutOutput {
    pub revoked: bool,
}
"#;

pub const APP_VALIDATION_MOD_RS: &str = r#"pub mod db;
pub mod sync;
pub mod username;
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

pub const APP_VALIDATION_USERNAME_RS: &str = r#"use std::borrow::Cow;
use validator::ValidationError;

fn err(code: &'static str, msg: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(msg))
}

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    let trimmed = value.trim();

    core_web::rules::required_trimmed(trimmed)
        .map_err(|_| err("required", "This field is required."))?;
    core_web::rules::alpha_dash(trimmed)
        .map_err(|_| err("alpha_dash", "Only lowercase letters, numbers, '-' and '_' are allowed."))?;

    if trimmed != trimmed.to_ascii_lowercase() {
        return Err(err(
            "lowercase",
            "Username must be lowercase.",
        ));
    }

    Ok(())
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
use tower_http::services::{ServeDir, ServeFile};

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

    // Admin SPA: /admin/* → public/admin/index.html
    let admin_public = public_path.join("admin");
    let admin_index = admin_public.join("index.html");
    if admin_public.is_dir() && admin_index.is_file() {
        router = router.nest_service(
            "/admin",
            ServeDir::new(&admin_public).fallback(ServeFile::new(&admin_index)),
        );
    }

    // User SPA: everything else → public/index.html (existing logic)
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
use core_db::infra::storage::Storage;
use core_web::datatable::DataTableEmailExportManager;

#[derive(Clone)]
pub struct AppApiState {
    pub db: sqlx::PgPool,
    pub auth: core_config::AuthSettings,
    pub storage: Arc<dyn Storage>,
    pub mailer: Arc<core_mailer::Mailer>,
    pub datatable_registry: Arc<DataTableRegistry>,
    pub datatable_async_exports: Arc<DataTableAsyncExportManager>,
    pub datatable_email_exports: Arc<DataTableEmailExportManager>,
    pub datatable_default_per_page: i64,
    pub datatable_unknown_filter_mode: DataTableUnknownFilterMode,
    pub datatable_export_link_ttl_secs: u64,
    pub app_timezone: String,
}

impl AppApiState {
    pub fn new(ctx: &BootContext) -> anyhow::Result<Self> {
        let mut datatable_registry = DataTableRegistry::new();
        crate::internal::datatables::register_all_generated_datatables(&mut datatable_registry, &ctx.db);
        datatable_registry.register_as(
            "admin.account",
            crate::internal::datatables::app_admin_datatable(ctx.db.clone()),
        );

        let datatable_registry = Arc::new(datatable_registry);
        let datatable_async_exports =
            Arc::new(DataTableAsyncExportManager::new(datatable_registry.clone()));

        Ok(Self {
            db: ctx.db.clone(),
            auth: ctx.settings.auth.clone(),
            storage: ctx.storage.clone(),
            mailer: ctx.mailer.clone(),
            datatable_registry,
            datatable_async_exports,
            datatable_email_exports: Arc::new(DataTableEmailExportManager::new()),
            datatable_default_per_page: ctx.settings.app.default_per_page as i64,
            datatable_unknown_filter_mode: map_unknown_filter_mode(
                ctx.settings.app.datatable_unknown_filter_mode,
            ),
            datatable_export_link_ttl_secs: ctx.settings.app.datatable_export_link_ttl_secs,
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
use core_datatable::{DataTableActor, DataTableAsyncExportManager, DataTableContext, DataTableRegistry};
use core_db::infra::storage::Storage;
use core_web::datatable::{
    DataTableEmailExportManager, DataTableRouteOptions, DataTableRouteState,
};
use core_web::auth::Guard;
use core_web::openapi::ApiRouter;
use serde_json::Value;

use generated::guards::AdminGuard;

use crate::contracts::datatable::admin::admin::AdminAdminDataTableContract;
use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    core_web::datatable::routes_for_scoped_contract_with_options(
        "/datatable/admin",
        state,
        AdminAdminDataTableContract,
        DataTableRouteOptions {
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

    fn datatable_storage(&self) -> &Arc<dyn Storage> {
        &self.storage
    }

    fn datatable_mailer(&self) -> &Arc<core_mailer::Mailer> {
        &self.mailer
    }

    fn datatable_email_exports(&self) -> &Arc<DataTableEmailExportManager> {
        &self.datatable_email_exports
    }

    fn datatable_export_link_ttl_secs(&self) -> u64 {
        self.datatable_export_link_ttl_secs
    }

    async fn datatable_context(&self, headers: &HeaderMap) -> DataTableContext {
        let actor = build_admin_actor(&self.db, headers).await;
        DataTableContext {
            default_per_page: self.datatable_default_per_page,
            app_timezone: self.app_timezone.clone(),
            user_timezone: core_web::utils::datatable::parse_timezone_from_headers(headers),
            actor,
            unknown_filter_mode: self.datatable_unknown_filter_mode,
        }
    }
}

async fn build_admin_actor(db: &sqlx::PgPool, headers: &HeaderMap) -> Option<DataTableActor> {
    let token = core_web::auth::extract_bearer_token(headers)?;
    let auth = core_web::auth::authenticate_token::<AdminGuard>(db, &token)
        .await
        .ok()?;

    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        "admin_type".to_string(),
        Value::String(auth.user.admin_type.as_str().to_string()),
    );

    Some(DataTableActor {
        id: auth.subject_id.clone(),
        guard: Some(AdminGuard::name().to_string()),
        roles: Vec::new(),
        permissions: auth.abilities,
        attributes,
    })
}
"#;

pub const APP_INTERNAL_API_V1_MOD_RS: &str = r#"use axum::middleware::from_fn_with_state;
use core_web::openapi::{
    aide::axum::routing::get_with,
    ApiRouter,
};

use crate::internal::api::{datatable, state::AppApiState};

mod admin;
mod admin_auth;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/user", user_router())
        .nest("/admin", admin_router(state))
}

fn user_router() -> ApiRouter {
    ApiRouter::new().api_route(
        "/health",
        get_with(user_health, |op| op.summary("User health").tag("User system")),
    )
}

fn admin_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/auth", admin_auth::router(state.clone()))
        .merge(admin_guarded_router(state))
}

fn admin_guarded_router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/health",
            get_with(admin_health, |op| op.summary("Admin health").tag("Admin system")),
        )
        .nest("/admins", admin::router(state.clone()))
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

pub const APP_INTERNAL_API_V1_ADMIN_RS: &str = r#"use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::{AsyncContractJson, ContractJson},
    error::AppError,
    extract::{validation::transform_validation_errors, AsyncValidate},
    openapi::{
        with_permission_check_delete_with, with_permission_check_get_with,
        with_permission_check_patch_with, with_permission_check_post_with, ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};

use crate::{
    contracts::api::v1::admin::{
        AdminDeleteOutput, AdminOutput, CreateAdminInput, UpdateAdminInput,
    },
    internal::{api::state::AppApiState, workflows::admin as workflow},
};

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Create admin").tag("Admin Account"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminRead.as_str(), Permission::AdminManage.as_str()],
                |op| op.summary("Get admin detail").tag("Admin Account"),
            )
            .merge(with_permission_check_patch_with(
                update,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Update admin").tag("Admin Account"),
            ))
            .merge(with_permission_check_delete_with(
                remove,
                AdminGuard,
                PermissionMode::Any,
                [Permission::AdminManage.as_str()],
                |op| op.summary("Delete admin").tag("Admin Account"),
            )),
        )
        .with_state(state)
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(AdminOutput::from(admin), &t("Admin loaded")))
}

async fn create(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: AsyncContractJson<CreateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let admin = workflow::create(&state, &auth, req.0).await?;
    Ok(ApiResponse::success(AdminOutput::from(admin), &t("Admin created")))
}

async fn update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
    req: ContractJson<UpdateAdminInput>,
) -> Result<ApiResponse<AdminOutput>, AppError> {
    let req = validate_update_input(&state, id, req.0).await?;
    let admin = workflow::update(&state, &auth, id, req).await?;
    Ok(ApiResponse::success(AdminOutput::from(admin), &t("Admin updated")))
}

async fn remove(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<AdminDeleteOutput>, AppError> {
    workflow::remove(&state, &auth, id).await?;
    Ok(ApiResponse::success(
        AdminDeleteOutput { deleted: true },
        &t("Admin deleted"),
    ))
}

async fn validate_update_input(
    state: &AppApiState,
    id: i64,
    req: UpdateAdminInput,
) -> Result<UpdateAdminInput, AppError> {
    let req = req.with_target_id(id);
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    Ok(req)
}
"#;

pub const APP_INTERNAL_API_V1_ADMIN_AUTH_RS: &str = r#"use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    middleware::from_fn_with_state,
};
use core_i18n::t;
use core_web::{
    auth::{self, AuthClientType, AuthUser, Guard},
    contracts::ContractJson,
    error::AppError,
    extract::request_headers::RequestHeaders,
    openapi::{
        aide::axum::routing::{get_with, patch_with, post_with},
        ApiRouter,
    },
    response::ApiResponse,
    utils::cookie,
};
use generated::guards::AdminGuard;
use std::ops::Deref;
use time::Duration;
use tower_cookies::Cookies;

use crate::{
    contracts::api::v1::admin_auth::{
        AdminAuthOutput, AdminLoginInput, AdminLogoutInput, AdminLogoutOutput, AdminMeOutput,
        AdminPasswordUpdateInput, AdminPasswordUpdateOutput, AdminProfileUpdateInput,
        AdminProfileUpdateOutput, AdminRefreshInput,
    },
    internal::{api::state::AppApiState, workflows::admin_auth as workflow},
};

const REFRESH_COOKIE_PATH: &str = "/api/v1/admin/auth";

#[derive(Debug, Clone)]
struct RequestCookies(Cookies);

impl Deref for RequestCookies {
    type Target = Cookies;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequestCookies
where
    S: Send + Sync,
{
    type Rejection = <Cookies as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await?;
        Ok(Self(cookies))
    }
}

impl core_web::openapi::aide::OperationInput for RequestCookies {}

pub fn router(state: AppApiState) -> ApiRouter {
    let protected = ApiRouter::new()
        .api_route(
            "/me",
            get_with(me, |op| {
                op.summary("Get current admin profile")
                    .tag("Admin Authentication")
            }),
        )
        .api_route(
            "/logout",
            post_with(logout, |op| op.summary("Logout admin").tag("Admin Authentication")),
        )
        .api_route(
            "/profile_update",
            patch_with(profile_update, |op| {
                op.summary("Update own profile")
                    .tag("Admin Authentication")
            }),
        )
        .api_route(
            "/password_update",
            patch_with(password_update, |op| {
                op.summary("Update own password")
                    .tag("Admin Authentication")
            }),
        )
        .layer(from_fn_with_state(
            state.clone(),
            crate::internal::middleware::auth::require_admin,
        ));

    ApiRouter::new()
        .api_route(
            "/login",
            post_with(login, |op| op.summary("Login admin").tag("Admin Authentication")),
        )
        .api_route(
            "/refresh",
            post_with(refresh, |op| {
                op.summary("Refresh admin access token")
                    .tag("Admin Authentication")
            }),
        )
        .merge(protected)
        .with_state(state)
}

async fn login(
    State(state): State<AppApiState>,
    cookies: RequestCookies,
    req: ContractJson<AdminLoginInput>,
) -> Result<ApiResponse<AdminAuthOutput>, AppError> {
    let req = req.0;
    let (_admin, tokens) = workflow::login(&state, &req.username, &req.password).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Login successful")))
}

async fn refresh(
    State(state): State<AppApiState>,
    headers: RequestHeaders,
    cookies: RequestCookies,
    req: ContractJson<AdminRefreshInput>,
) -> Result<ApiResponse<AdminAuthOutput>, AppError> {
    let req = req.0;
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        AdminGuard::name(),
        req.client_type,
        req.refresh_token.as_deref(),
    )
    .ok_or_else(|| AppError::BadRequest(t("Missing refresh token")))?;

    let tokens = workflow::refresh(&state, &refresh_token).await?;
    let output = to_auth_output(&state, &cookies, req.client_type, tokens);
    Ok(ApiResponse::success(output, &t("Token refreshed")))
}

async fn logout(
    State(state): State<AppApiState>,
    headers: RequestHeaders,
    cookies: RequestCookies,
    _auth: AuthUser<AdminGuard>,
    req: ContractJson<AdminLogoutInput>,
) -> Result<ApiResponse<AdminLogoutOutput>, AppError> {
    let req = req.0;
    let refresh_token = auth::extract_refresh_token_for_client(
        &headers,
        AdminGuard::name(),
        req.client_type,
        req.refresh_token.as_deref(),
    )
    .ok_or_else(|| AppError::BadRequest(t("Missing refresh token")))?;

    workflow::revoke_session(&state, &refresh_token).await?;

    if matches!(req.client_type, AuthClientType::Web) {
        cookie::remove_guard_refresh(&cookies, AdminGuard::name(), REFRESH_COOKIE_PATH);
    }

    Ok(ApiResponse::success(
        AdminLogoutOutput { revoked: true },
        &t("Logout successful"),
    ))
}

async fn me(auth: AuthUser<AdminGuard>) -> Result<ApiResponse<AdminMeOutput>, AppError> {
    let user = auth.user;
    Ok(ApiResponse::success(
        AdminMeOutput {
            id: user.id,
            username: user.username,
            email: user.email,
            name: user.name,
            admin_type: user.admin_type,
            scopes: auth.abilities,
        },
        &t("Profile loaded"),
    ))
}

async fn profile_update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: ContractJson<AdminProfileUpdateInput>,
) -> Result<ApiResponse<AdminProfileUpdateOutput>, AppError> {
    let req = req.0;
    let admin = workflow::profile_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        AdminProfileUpdateOutput {
            id: admin.id,
            username: admin.username,
            email: admin.email,
            name: admin.name,
            admin_type: admin.admin_type,
        },
        &t("Profile updated successfully"),
    ))
}

async fn password_update(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: ContractJson<AdminPasswordUpdateInput>,
) -> Result<ApiResponse<AdminPasswordUpdateOutput>, AppError> {
    let req = req.0;
    workflow::password_update(&state, auth.user.id, req).await?;
    Ok(ApiResponse::success(
        AdminPasswordUpdateOutput { updated: true },
        &t("Password updated successfully"),
    ))
}

fn to_auth_output(
    state: &AppApiState,
    cookies: &Cookies,
    client_type: AuthClientType,
    tokens: core_web::auth::IssuedTokenPair,
) -> AdminAuthOutput {
    match client_type {
        AuthClientType::Web => {
            if let Some(ttl) = refresh_cookie_ttl(state) {
                cookie::set_guard_refresh(
                    cookies,
                    AdminGuard::name(),
                    &tokens.refresh_token,
                    ttl,
                    REFRESH_COOKIE_PATH,
                );
            }

            AdminAuthOutput {
                token_type: "Bearer".to_string(),
                access_token: tokens.access_token,
                access_expires_at: tokens.access_expires_at,
                refresh_token: None,
                scopes: tokens.abilities,
            }
        }
        AuthClientType::Mobile => AdminAuthOutput {
            token_type: "Bearer".to_string(),
            access_token: tokens.access_token,
            access_expires_at: tokens.access_expires_at,
            refresh_token: Some(tokens.refresh_token),
            scopes: tokens.abilities,
        },
    }
}

fn refresh_cookie_ttl(state: &AppApiState) -> Option<Duration> {
    let days = state.auth.guard(AdminGuard::name())?.refresh_ttl_days;
    let days = i64::try_from(days).ok()?;
    Some(Duration::days(days))
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

pub const APP_INTERNAL_WORKFLOWS_MOD_RS: &str = r#"pub mod admin;
pub mod admin_auth;
"#;

pub const APP_INTERNAL_WORKFLOWS_ADMIN_RS: &str = r#"use core_db::common::sql::{
    DbConn, Op, generate_snowflake_i64,
};
use core_i18n::t;
use core_web::{auth::AuthUser, error::AppError};
use generated::{
    guards::AdminGuard,
    models::{Admin, AdminType, AdminView},
    permissions::Permission,
};

use crate::{
    contracts::api::v1::admin::{CreateAdminInput, UpdateAdminInput},
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<AdminView, AppError> {
    Admin::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn create(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    req: CreateAdminInput,
) -> Result<AdminView, AppError> {
    let username = req.username.trim().to_ascii_lowercase();

    let abilities = ensure_assignable_permissions(auth, &req.abilities)?;

    let mut insert = Admin::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_username(username)
        .set_name(req.name.trim().to_string())
        .set_admin_type(req.admin_type)
        .set_abilities(permissions_to_json(&abilities));

    if let Some(email) = normalize_optional_email(req.email) {
        insert = insert.set_email(Some(email));
    }

    let insert = insert.set_password(&req.password).map_err(AppError::from)?;
    insert.save().await.map_err(AppError::from)
}

pub async fn update(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    id: i64,
    req: UpdateAdminInput,
) -> Result<AdminView, AppError> {
    if auth.user.id == id {
        return Err(AppError::Forbidden(t(
            "You cannot update your own admin account here",
        )));
    }

    let existing = detail(state, id).await?;
    let mut update = Admin::new(DbConn::pool(&state.db), None).update().where_id(Op::Eq, id);
    let mut touched = false;

    if let Some(username) = req.username {
        let username = username.trim().to_ascii_lowercase();
        if username != existing.username {
            update = update.set_username(username);
            touched = true;
        }
    }

    if let Some(name) = req.name {
        update = update.set_name(name.trim().to_string());
        touched = true;
    }

    if let Some(email) = normalize_optional_email(req.email) {
        update = update.set_email(Some(email));
        touched = true;
    }

    if let Some(admin_type) = req.admin_type {
        update = update.set_admin_type(admin_type);
        touched = true;
    }

    if let Some(abilities) = req.abilities {
        let abilities = ensure_assignable_permissions(auth, &abilities)?;
        update = update.set_abilities(permissions_to_json(&abilities));
        touched = true;
    }

    if !touched {
        return Ok(existing);
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    detail(state, id).await
}

pub async fn remove(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    id: i64,
) -> Result<(), AppError> {
    if auth.user.id == id {
        return Err(AppError::Forbidden(t(
            "You cannot delete your own admin account here",
        )));
    }

    let affected = Admin::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }
    Ok(())
}

fn normalize_optional_email(email: Option<String>) -> Option<String> {
    email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn ensure_assignable_permissions(
    auth: &AuthUser<AdminGuard>,
    requested: &[Permission],
) -> Result<Vec<String>, AppError> {
    if matches!(auth.user.admin_type, AdminType::Admin)
        && requested
            .iter()
            .any(|permission| matches!(permission, Permission::AdminRead | Permission::AdminManage))
    {
        return Err(AppError::Forbidden(t(
            "Normal admin cannot assign admin.read or admin.manage",
        )));
    }

    let requested = requested
        .iter()
        .map(|permission| permission.as_str().to_string())
        .collect::<Vec<_>>();

    if matches!(auth.user.admin_type, AdminType::Developer | AdminType::SuperAdmin) {
        return Ok(requested);
    }

    if requested
        .iter()
        .all(|permission| auth.has_permission(permission.as_str()))
    {
        return Ok(requested);
    }

    Err(AppError::Forbidden(t(
        "Cannot assign permissions you do not have",
    )))
}

fn permissions_to_json(values: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        values
            .iter()
            .map(|value| serde_json::Value::String(value.clone()))
            .collect(),
    )
}
"#;

pub const APP_INTERNAL_WORKFLOWS_ADMIN_AUTH_RS: &str = r#"use core_db::common::{
    auth::hash::verify_password,
    sql::{DbConn, Op},
};
use core_i18n::t;
use core_web::{
    auth::{self, IssuedTokenPair, TokenScopeGrant},
    error::AppError,
};
use generated::{
    guards::AdminGuard,
    models::{Admin, AdminQuery, AdminType, AdminView},
    permissions::Permission,
};

use crate::contracts::api::v1::admin_auth::{AdminPasswordUpdateInput, AdminProfileUpdateInput};
use crate::internal::api::state::AppApiState;

pub fn resolve_scope_grant(admin: &AdminView) -> TokenScopeGrant {
    match admin.admin_type {
        AdminType::Developer | AdminType::SuperAdmin => TokenScopeGrant::Wildcard,
        AdminType::Admin => {
            let explicit = admin_permissions(admin);
            if explicit.is_empty() {
                TokenScopeGrant::AuthOnly
            } else {
                TokenScopeGrant::Explicit(explicit)
            }
        }
    }
}

fn admin_permissions(admin: &AdminView) -> Vec<String> {
    let mut out = Vec::new();

    if let Some(items) = admin.abilities.as_array() {
        for item in items {
            let Some(raw) = item.as_str() else {
                continue;
            };
            let value = raw.trim();
            if value.is_empty() {
                continue;
            }
            if value == "*" {
                out.push("*".to_string());
                continue;
            }
            if let Some(permission) = Permission::from_str(value) {
                out.push(permission.as_str().to_string());
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

pub async fn login(
    state: &AppApiState,
    username: &str,
    password: &str,
) -> Result<(AdminView, IssuedTokenPair), AppError> {
    let username = username.trim().to_ascii_lowercase();
    let admin = AdminQuery::new(DbConn::pool(&state.db), None)
        .where_username(Op::Eq, username)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::Unauthorized(t("Invalid credentials")))?;

    let valid = verify_password(password, &admin.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Invalid credentials")));
    }

    let scope_grant = resolve_scope_grant(&admin);
    let tokens = auth::issue_guard_session::<AdminGuard>(
        &state.db,
        &state.auth,
        admin.id,
        "admin-session",
        scope_grant,
    )
    .await
    .map_err(AppError::from)?;

    Ok((admin, tokens))
}

pub async fn refresh(state: &AppApiState, refresh_token: &str) -> Result<IssuedTokenPair, AppError> {
    auth::refresh_guard_session::<AdminGuard>(&state.db, &state.auth, refresh_token, "admin-session")
        .await
}

pub async fn revoke_session(state: &AppApiState, refresh_token: &str) -> Result<(), AppError> {
    auth::revoke_session_by_refresh_token::<AdminGuard>(&state.db, refresh_token).await
}

pub async fn profile_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminProfileUpdateInput,
) -> Result<AdminView, AppError> {
    let mut update = Admin::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, admin_id)
        .set_name(req.name.trim().to_string());

    if let Some(email) = req.email {
        let email = email.trim().to_ascii_lowercase();
        if !email.is_empty() {
            update = update.set_email(Some(email));
        }
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Admin::new(DbConn::pool(&state.db), None)
        .find(admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))
}

pub async fn password_update(
    state: &AppApiState,
    admin_id: i64,
    req: AdminPasswordUpdateInput,
) -> Result<(), AppError> {
    let admin = Admin::new(DbConn::pool(&state.db), None)
        .find(admin_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Admin not found")))?;

    let valid = verify_password(&req.current_password, &admin.password).map_err(AppError::from)?;
    if !valid {
        return Err(AppError::Unauthorized(t("Current password is incorrect")));
    }

    let update = Admin::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, admin_id)
        .set_password(&req.password)
        .map_err(AppError::from)?;

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Admin not found")));
    }

    Ok(())
}
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

pub const APP_INTERNAL_DATATABLES_ADMIN_RS: &str = r#"use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::Op;
use core_web::authz::{has_required_permissions, PermissionMode};
use generated::{
    models::{AdminDataTable, AdminDataTableConfig, AdminDataTableHooks, AdminQuery, AdminType},
    permissions::Permission,
};

#[derive(Default, Clone)]
pub struct AdminDataTableAppHooks;

impl AdminDataTableHooks for AdminDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: AdminQuery<'db>,
        _input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> AdminQuery<'db> {
        let Some(actor) = ctx.actor.as_ref() else {
            return query.where_id(Op::Eq, -1);
        };

        let admin_type = actor
            .attributes
            .get("admin_type")
            .and_then(|value| value.as_str())
            .and_then(parse_admin_type);

        match admin_type {
            Some(AdminType::Developer) => query,
            Some(AdminType::SuperAdmin) => query.where_admin_type(Op::Ne, AdminType::Developer),
            Some(AdminType::Admin) => query.where_admin_type(Op::Eq, AdminType::Admin),
            None => query.where_id(Op::Eq, -1),
        }
    }

    fn authorize(&self, _input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        Ok(has_required_permissions(
            &actor.permissions,
            &[Permission::AdminRead.as_str(), Permission::AdminManage.as_str()],
            PermissionMode::Any,
        ))
    }
}

fn parse_admin_type(value: &str) -> Option<AdminType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "developer" => Some(AdminType::Developer),
        "superadmin" => Some(AdminType::SuperAdmin),
        "admin" => Some(AdminType::Admin),
        _ => None,
    }
}

pub type AppAdminDataTable = AdminDataTable<AdminDataTableAppHooks>;

pub fn app_admin_datatable(db: sqlx::PgPool) -> AppAdminDataTable {
    AdminDataTable::new(db).with_hooks(AdminDataTableAppHooks::default())
}

pub fn app_admin_datatable_with_config(
    db: sqlx::PgPool,
    config: AdminDataTableConfig,
) -> AppAdminDataTable {
    AdminDataTable::new(db)
        .with_hooks(AdminDataTableAppHooks::default())
        .with_config(config)
}

pub fn register_admin_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register(app_admin_datatable(db));
}
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
    common::auth::hash::hash_password,
    seeder::Seeder,
};

#[derive(Debug, Default)]
pub struct AdminBootstrapSeeder;

#[async_trait]
impl Seeder for AdminBootstrapSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        if should_skip_in_env() {
            return Ok(());
        }

        upsert_admin(
            db,
            &env_or("SEED_ADMIN_DEVELOPER_USERNAME", "developer"),
            optional_env("SEED_ADMIN_DEVELOPER_EMAIL"),
            &env_or("SEED_ADMIN_DEVELOPER_PASSWORD", "password123"),
            &env_or("SEED_ADMIN_DEVELOPER_NAME", "Developer"),
            "developer",
        )
        .await?;

        upsert_admin(
            db,
            &env_or("SEED_ADMIN_SUPERADMIN_USERNAME", "superadmin"),
            optional_env("SEED_ADMIN_SUPERADMIN_EMAIL"),
            &env_or("SEED_ADMIN_SUPERADMIN_PASSWORD", "password123"),
            &env_or("SEED_ADMIN_SUPERADMIN_NAME", "Super Admin"),
            "superadmin",
        )
        .await?;

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

    let raw = env_or("SEED_ADMIN_BOOTSTRAP_IN_PROD", "");
    !is_truthy(&raw)
}

fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "y"
    )
}

fn env_or(key: &str, default: &str) -> String {
    if let Ok(value) = std::env::var(key) {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }
    default.to_string()
}

fn optional_env(key: &str) -> Option<String> {
    if let Ok(value) = std::env::var(key) {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_ascii_lowercase());
        }
    }
    None
}

async fn upsert_admin(
    db: &sqlx::PgPool,
    username: &str,
    email: Option<String>,
    password_plain: &str,
    name: &str,
    admin_type: &str,
) -> anyhow::Result<i64> {
    let password = hash_password(password_plain)?;
    let id_to_insert = core_db::common::sql::generate_snowflake_i64();
    let username = username.trim().to_ascii_lowercase();

    let id = sqlx::query_scalar::<_, i64>(
        "\n        INSERT INTO admin (id, username, email, password, name, admin_type, abilities)\n        VALUES ($1, $2, $3, $4, $5, $6, '[]'::jsonb)\n        ON CONFLICT (username) DO UPDATE\n        SET\n            email = EXCLUDED.email,\n            password = EXCLUDED.password,\n            name = EXCLUDED.name,\n            admin_type = EXCLUDED.admin_type,\n            updated_at = NOW()\n        RETURNING id\n        ",
    )
    .bind(id_to_insert)
    .bind(username)
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
    db_gen::generate_auth(&cfgs, &schema, &guards_out).expect("Failed to gen auth");
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

// ── Agent guideline files (split per folder) ────────────────────────

pub const ROOT_AGENTS_MD: &str = r#"# Rustforge Project

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

1. `frontend/vite.config.{name}.ts` — set `base: "/{name}/"`, unique port, `outDir: "../public/{name}"`
2. `frontend/{name}.html` + `frontend/src/{name}/{main.tsx, App.tsx, app.css}`
3. Add `dev:{name}` and `build:{name}` to `package.json` scripts
4. Update `build` script ordering (nested portals before root)
5. Add `nest_service("/{name}", ...)` in `build_router` (before the user SPA catch-all)

## New Feature Checklist

1. Schema → `app/schemas/{domain}.toml`
2. Migration → `migrations/{number}_{name}.sql`
3. Permissions → `app/permissions.toml`
4. Contracts → `app/src/contracts/api/v1/{domain}.rs` (add `#[derive(TS)]` for frontend types)
5. Workflow → `app/src/internal/workflows/{domain}.rs`
6. Handler → `app/src/internal/api/v1/{domain}.rs`
7. Wire routes → `app/src/internal/api/v1/mod.rs`
8. Module declarations → add `mod`/`pub mod` in relevant `mod.rs`
9. Translations → add keys to all `i18n/*.json` files
10. `cargo check` to trigger code generation
11. Run `make gen-types` to regenerate frontend TypeScript types from contracts
"#;

pub const CONTRACTS_AGENTS_MD: &str = r#"# Contracts

Request/response DTOs that define the API surface. Lives in `contracts/api/v1/` (versioned), `contracts/datatable/`, and `contracts/types/`.

## Input Structs — `#[rustforge_contract]`

Auto-injects `Debug, Clone, Deserialize, Validate, JsonSchema`. Use `#[rf(...)]` for validation rules.

```rust
use core_web::contracts::rustforge_contract;

#[rustforge_contract]
pub struct CreateArticleInput {
    #[rf(length(min = 3, max = 255))]
    #[rf(alpha_dash)]
    pub slug: String,

    #[rf(length(min = 1, max = 1000))]
    pub title: String,

    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,

    #[rf(nested)]
    pub metadata: MetadataInput,
}
```

## `#[rf(...)]` Rules

| Rule | Usage |
|------|-------|
| `length(min, max)` | `#[rf(length(min = 3, max = 64))]` |
| `range(min, max)` | `#[rf(range(min = 1, max = 100))]` |
| `email` | `#[rf(email)]` |
| `url` | `#[rf(url)]` |
| `alpha_dash` | letters, digits, `_`, `-` |
| `one_of(...)` | `#[rf(one_of("a", "b", "c"))]` |
| `none_of(...)` | `#[rf(none_of("x", "y"))]` |
| `regex(pattern)` | `#[rf(regex(pattern = r"^\d{4}$"))]` |
| `contains(pattern)` | `#[rf(contains(pattern = "@"))]` |
| `does_not_contain(pattern)` | `#[rf(does_not_contain(pattern = "banned"))]` |
| `must_match(other)` | `#[rf(must_match(other = "password_confirmation"))]` |
| `nested` | validate nested struct recursively |
| `date(format)` | `#[rf(date(format = "%Y-%m-%d"))]` |
| `phonenumber(field)` | `#[rf(phonenumber(field = "country_iso2"))]` |
| `async_unique(...)` | `#[rf(async_unique(table = "user", column = "email"))]` |
| `async_exists(...)` | `#[rf(async_exists(table = "role", column = "id"))]` |
| `async_not_exists(...)` | `#[rf(async_not_exists(table = "banned", column = "email"))]` |
| `openapi(...)` | `#[rf(openapi(description = "...", example = "..."))]` |

### Async unique with modifiers

```rust
#[rf(async_unique(
    table = "admin", column = "username",
    ignore(column = "id", field = "__target_id"),
    where_null(column = "deleted_at")
))]
```

### Update contracts with target ID for ignore

```rust
#[rustforge_contract]
pub struct UpdateArticleInput {
    #[serde(skip, default)]
    __target_id: i64,

    #[serde(default)]
    #[rf(length(min = 3, max = 255))]
    #[rf(async_unique(table = "article", column = "slug", ignore(column = "id", field = "__target_id")))]
    pub slug: Option<String>,
}

impl UpdateArticleInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}
```

## Output Structs — manual derives (no macro)

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ArticleOutput {
    pub id: i64,
    pub title: String,
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
}

impl From<generated::models::ArticleView> for ArticleOutput {
    fn from(v: generated::models::ArticleView) -> Self {
        Self { id: v.id, title: v.title, created_at: v.created_at }
    }
}
```

Use `#[schemars(with = "String")]` for types that don't implement `JsonSchema` (e.g. `time::OffsetDateTime`).

## Reusable String-Wrapper Types

For validation rules shared across contracts, define in `contracts/types/`:

```rust
use core_web::contracts::rustforge_string_rule_type;

rustforge_string_rule_type! {
    pub struct EmailAddress {
        #[rf(email)]
        #[rf(openapi(description = "Valid email", example = "user@example.com"))]
    }
}
```

Use as field type with `#[rf(nested)]`:
```rust
#[rf(nested)]
pub email: EmailAddress,
```

## TypeScript Type Generation

Contract structs are auto-exported to TypeScript via `ts-rs`. Add `#[derive(TS)]` alongside existing derives.

### Input structs (with `#[rustforge_contract]`)

```rust
use ts_rs::TS;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CreateArticleInput {
    #[rf(length(min = 1, max = 255))]
    pub title: String,

    #[ts(type = "ArticleStatus")]           // generated enum — override type
    pub status: ArticleStatus,

    #[ts(type = "string")]                  // newtype wrapper — flatten to string
    #[rf(nested)]
    pub slug: SlugString,

    #[serde(default)]
    pub tags: Vec<String>,                  // ts-rs handles Vec<String> natively
}
```

### Output structs

```rust
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ArticleOutput {
    pub id: i64,
    pub title: String,
    #[ts(type = "string")]                  // OffsetDateTime → string
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
}
```

### Registering in `export-types.rs`

After adding `#[derive(TS)]` to your structs, register them in `app/src/bin/export-types.rs`:

```rust
// Add a new TsFile block:
{
    use app::contracts::api::v1::article::*;
    files.push(TsFile {
        rel_path: "admin/types/article.ts",
        imports: &["import type { ArticleStatus } from \"./enums\";"],
        definitions: vec![
            CreateArticleInput::export_to_string().expect("CreateArticleInput"),
            ArticleOutput::export_to_string().expect("ArticleOutput"),
        ],
    });
}
```

Then update the barrel `frontend/src/admin/types/index.ts` to re-export and run `make gen-types`.

### Conventions

- Only **serde-visible** fields are exported (fields with `#[serde(skip)]` are excluded)
- Use `#[ts(type = "TypeName")]` for types that don't derive `TS` (generated enums, framework types, newtypes)
- Use `#[ts(type = "string")]` for `time::OffsetDateTime` and string newtypes
- `Option<T>` becomes `T | null` automatically
- `Vec<T>` becomes `T[]` automatically
- `#[serde(default)]` fields become optional in TypeScript (with `serde-compat` feature)
"#;

pub const API_AGENTS_MD: &str = r#"# API Handlers

Route handlers in `api/v1/`. Handlers are **thin** — parse input, call workflow, wrap in response.

## Handler Pattern

```rust
use axum::extract::{Path, State};
use core_i18n::t;
use core_web::{
    auth::AuthUser,
    authz::PermissionMode,
    contracts::{AsyncContractJson, ContractJson},
    error::AppError,
    openapi::{
        with_permission_check_get_with, with_permission_check_post_with,
        with_permission_check_patch_with, with_permission_check_delete_with,
        ApiRouter,
    },
    response::ApiResponse,
};
use generated::{guards::AdminGuard, permissions::Permission};
use crate::internal::api::state::AppApiState;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            with_permission_check_post_with(
                create, AdminGuard, PermissionMode::Any,
                [Permission::ArticleManage.as_str()],
                |op| op.summary("Create article").tag("Articles"),
            ),
        )
        .api_route(
            "/{id}",
            with_permission_check_get_with(
                detail, AdminGuard, PermissionMode::Any,
                [Permission::ArticleRead.as_str()],
                |op| op.summary("Get article").tag("Articles"),
            ),
        )
        .with_state(state)
}

async fn create(
    State(state): State<AppApiState>,
    auth: AuthUser<AdminGuard>,
    req: AsyncContractJson<CreateArticleInput>,
) -> Result<ApiResponse<ArticleOutput>, AppError> {
    let article = workflow::create(&state, &auth, req.0).await?;
    Ok(ApiResponse::success(ArticleOutput::from(article), &t("Article created")))
}

async fn detail(
    State(state): State<AppApiState>,
    _auth: AuthUser<AdminGuard>,
    Path(id): Path<i64>,
) -> Result<ApiResponse<ArticleOutput>, AppError> {
    let article = workflow::detail(&state, id).await?;
    Ok(ApiResponse::success(ArticleOutput::from(article), &t("Article loaded")))
}
```

## Extractors

| Extractor | When to use |
|-----------|-------------|
| `ContractJson<T>` | Sync validation only |
| `AsyncContractJson<T>` | Has `async_unique`/`async_exists` rules |

For update with async validation, validate manually:
```rust
async fn update(
    State(state): State<AppApiState>,
    Path(id): Path<i64>,
    req: ContractJson<UpdateInput>,
) -> Result<ApiResponse<Output>, AppError> {
    let req = req.0.with_target_id(id);
    if let Err(e) = req.validate_async(&state.db).await {
        return Err(AppError::Validation {
            message: t("Validation failed"),
            errors: transform_validation_errors(e),
        });
    }
    // ...
}
```

## Router Wiring

Register new domain routers in `api/v1/mod.rs`:
```rust
mod article;

pub fn router(state: AppApiState) -> ApiRouter {
    ApiRouter::new()
        .nest("/articles", article::router(state.clone()))
        // ...
}
```

Guarded routes use middleware layer:
```rust
.layer(from_fn_with_state(state, crate::internal::middleware::auth::require_admin))
```

## Auth in Handlers

```rust
// Extract user
auth: AuthUser<AdminGuard>

// Permission check
use core_web::authz::{PermissionMode, ensure_permissions};
ensure_permissions(&auth, PermissionMode::Any, &["article.read"])?;

// Direct check
auth.has_permission("article.manage")
```

## State

`AppApiState` in `state.rs` holds `db`, `auth`, `storage`, `mailer`, registries. Extend it when adding new shared resources.
"#;

pub const WORKFLOWS_AGENTS_MD: &str = r#"# Workflows

Business logic functions. One file per domain. Handlers call these — keep DB queries, permission checks, and orchestration here.

## Pattern

```rust
use core_db::common::sql::{DbConn, Op, generate_snowflake_i64};
use core_i18n::t;
use core_web::{auth::AuthUser, error::AppError};
use generated::{guards::AdminGuard, models::{Article, ArticleView, ArticleQuery}};
use crate::internal::api::state::AppApiState;

pub async fn detail(state: &AppApiState, id: i64) -> Result<ArticleView, AppError> {
    Article::new(DbConn::pool(&state.db), None)
        .find(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Article not found")))
}

pub async fn create(
    state: &AppApiState,
    auth: &AuthUser<AdminGuard>,
    req: CreateArticleInput,
) -> Result<ArticleView, AppError> {
    Article::new(DbConn::pool(&state.db), None)
        .insert()
        .set_id(generate_snowflake_i64())
        .set_title(req.title.trim().to_string())
        .set_slug(req.slug.trim().to_ascii_lowercase())
        .save()
        .await
        .map_err(AppError::from)
}

pub async fn update(state: &AppApiState, id: i64, req: UpdateArticleInput) -> Result<ArticleView, AppError> {
    let mut update = Article::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id);

    if let Some(title) = req.title {
        update = update.set_title(title.trim().to_string());
    }

    let affected = update.save().await.map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Article not found")));
    }

    detail(state, id).await
}

pub async fn remove(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = Article::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;
    if affected == 0 {
        return Err(AppError::NotFound(t("Article not found")));
    }
    Ok(())
}
```

## Generated Model API

| Operation | Code |
|-----------|------|
| Create handle | `Model::new(DbConn::pool(&db), None)` |
| Insert | `.insert().set_field(val).save()` → `ModelView` |
| Update | `.update().where_id(Op::Eq, id).set_field(val).save()` → affected rows |
| Delete | `.delete(id)` → affected rows (soft-delete if enabled) |
| Find by PK | `.find(id)` → `Option<ModelView>` |
| Query | `ModelQuery::new(...).where_field(Op::Eq, val).first()` → `Option<ModelView>` |
| Hashed field | `.set_password(&plain_text).map_err(AppError::from)?` (returns Result) |

IDs use snowflake: `generate_snowflake_i64()`.
"#;

pub const JOBS_AGENTS_MD: &str = r#"# Background Jobs

Define job structs, register them in this module, and dispatch from workflows.

## Define a Job

```rust
use async_trait::async_trait;
use core_jobs::{Job, JobContext};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendWelcomeEmailJob {
    pub user_id: i64,
    pub email: String,
}

#[async_trait]
impl Job for SendWelcomeEmailJob {
    const NAME: &'static str = "SendWelcomeEmail";
    const QUEUE: &'static str = "emails";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // ctx.db, ctx.redis, ctx.settings available
        Ok(())
    }

    fn max_retries(&self) -> u32 { 3 }
}
```

## Register

In `jobs/mod.rs`:
```rust
pub fn register_jobs(worker: &mut Worker) {
    worker.register::<SendWelcomeEmailJob>();
}

pub fn register_schedules(scheduler: &mut Scheduler) {
    scheduler.cron::<DailyCleanupJob>("0 2 * * *");
}
```

## Dispatch

```rust
let job = SendWelcomeEmailJob { user_id: 1, email: "a@b.com".into() };
job.dispatch(&state.queue).await?;
```
"#;

pub const MIDDLEWARE_AGENTS_MD: &str = r#"# Middleware

Custom middleware functions. Framework applies standard stack (CORS, rate limit, timeout, compression, auth headers) automatically.

## Auth Middleware Pattern

```rust
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use core_web::{auth, error::AppError};
use generated::guards::AdminGuard;

pub async fn require_admin<B>(
    State(state): State<AppApiState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let token = auth::extract_bearer_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized(t("Missing token")))?;
    let auth_user = auth::authenticate_token::<AdminGuard>(&state.db, &token).await?;
    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}
```

Apply to routes via `from_fn_with_state(state, require_admin)`.
"#;

pub const DATATABLES_AGENTS_MD: &str = r#"# Datatables

Server-side datatable executors. Generated stubs come from `db-gen`; custom datatables are registered manually in `state.rs`.

## Custom Datatable

Override or extend generated datatables here. Registration happens in `AppApiState::new()`:

```rust
datatable_registry.register_as("article.list", custom_article_datatable(ctx.db.clone()));
```

## Datatable Contract

Define query/export contracts in `contracts/datatable/{domain}/`. They specify filters, columns, and export formats available to the datatable.
"#;

pub const REALTIME_AGENTS_MD: &str = r#"# Realtime

WebSocket channel policies and authorizers. Channels are configured in `app/configs.toml`:

```toml
[realtime.channels.notifications]
enabled = true
guard = "admin"
presence_enabled = true
```

## Channel Policy

Implement subscribe/publish authorization logic here for channels that need custom access control.
"#;

pub const VALIDATION_AGENTS_MD: &str = r#"# Validation

Custom validation rules — both sync and async (DB).

## Sync Validators

Return `Result<(), ValidationError>`. Use in contracts with `#[validate(custom(function = "path"))]`.

```rust
use std::borrow::Cow;
use validator::ValidationError;

pub fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if value.contains("--") {
        let mut err = ValidationError::new("slug");
        err.message = Some(Cow::from("Slug cannot contain consecutive hyphens"));
        return Err(err);
    }
    Ok(())
}
```

## Async Validators (DB)

For `async_unique` / `async_exists` rules, the `#[rf(...)]` macro generates async validation automatically. For custom async checks, implement `AsyncValidate`:

```rust
use core_web::extract::AsyncValidate;

#[async_trait]
impl AsyncValidate for MyInput {
    async fn validate_async(&self, db: &sqlx::PgPool) -> Result<(), validator::ValidationErrors> {
        // Custom DB checks
        Ok(())
    }
}
```
"#;

pub const SEEDS_AGENTS_MD: &str = r#"# Seeds

Database seeders for initial/test data. Implement the `Seeder` trait.

```rust
use async_trait::async_trait;
use core_db::seeder::Seeder;

pub struct ArticleSeeder;

#[async_trait]
impl Seeder for ArticleSeeder {
    fn name(&self) -> &str { "ArticleSeeder" }

    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        // Insert seed data
        Ok(())
    }
}
```

Register in `seeds/mod.rs` and pass to `bootstrap::console::start_console`.

Run: `./console db seed`
"#;

// ── Frontend template files ──────────────────────────────

pub const FRONTEND_PACKAGE_JSON: &str = r#"{
  "name": "frontend",
  "private": true,
  "type": "module",
  "scripts": {
    "dev:user": "vite --config vite.config.user.ts",
    "dev:admin": "vite --config vite.config.admin.ts",
    "build:user": "vite build --config vite.config.user.ts",
    "build:admin": "vite build --config vite.config.admin.ts",
    "build": "rm -rf ../public && npm run build:admin && npm run build:user",
    "preview:user": "vite preview --config vite.config.user.ts",
    "preview:admin": "vite preview --config vite.config.admin.ts"
  },
  "dependencies": {
    "axios": "^1.7.0",
    "i18next": "^24.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "react-i18next": "^15.0.0",
    "react-router-dom": "^7.0.0",
    "zustand": "^5.0.0"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.5.0",
    "tailwindcss": "^4.0.0",
    "typescript": "~5.7.0",
    "vite": "^6.0.0"
  }
}
"#;

pub const FRONTEND_VITE_CONFIG_USER_TS: &str = r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  root: ".",
  base: "/",
  build: {
    outDir: "../public",
    emptyOutDir: false,
    rollupOptions: {
      input: "user.html",
    },
  },
  // Rename user.html → index.html in the output so the Rust SPA
  // fallback (which looks for public/index.html) works unchanged.
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === "html") return filename;
      return "/" + filename;
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
"#;

pub const FRONTEND_VITE_CONFIG_ADMIN_TS: &str = r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  root: ".",
  base: "/admin/",
  build: {
    outDir: "../public/admin",
    emptyOutDir: true,
    rollupOptions: {
      input: "admin.html",
    },
  },
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === "html") return filename;
      return "/admin/" + filename;
    },
  },
  server: {
    port: 5174,
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
"#;

pub const FRONTEND_TSCONFIG_JSON: &str = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true
  },
  "include": ["src"]
}
"#;

pub const FRONTEND_TSCONFIG_NODE_JSON: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true
  },
  "include": ["vite.config.*.ts"]
}
"#;

pub const FRONTEND_POSTCSS_CONFIG_JS: &str = r#"export default {
  plugins: {
    "@tailwindcss/postcss": {},
    autoprefixer: {},
  },
};
"#;

pub const FRONTEND_USER_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Starter</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/user/main.tsx"></script>
  </body>
</html>
"#;

pub const FRONTEND_ADMIN_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Admin</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/admin/main.tsx"></script>
  </body>
</html>
"#;

pub const FRONTEND_SRC_USER_MAIN_TSX: &str = r#"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "../shared/i18n";
import App from "./App";
import "./app.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>,
);
"#;

pub const FRONTEND_SRC_USER_APP_TSX: &str = r#"import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "../shared/ProtectedRoute";
import { useAuthStore } from "./stores/auth";

function DashboardPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold tracking-tight">Rustforge Starter</h1>
        <p className="mt-2 text-lg text-muted">User Portal</p>
      </div>
    </div>
  );
}

function LoginPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold tracking-tight">Login</h1>
        <p className="mt-2 text-lg text-muted">Build your login form here.</p>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route path="/*" element={<DashboardPage />} />
      </Route>
    </Routes>
  );
}
"#;

pub const FRONTEND_SRC_USER_APP_CSS: &str = r#"@import "tailwindcss";

@theme {
  --color-background: #f8fafc;
  --color-foreground: #0f172a;
  --color-muted: #64748b;
  --color-muted-foreground: #94a3b8;
  --color-surface: #ffffff;
  --color-surface-hover: #f1f5f9;
  --color-surface-active: #e2e8f0;
  --color-primary: #2563eb;
  --color-primary-hover: #1d4ed8;
  --color-primary-foreground: #ffffff;
  --color-border: #e2e8f0;
  --color-border-hover: #cbd5e1;
  --color-input: #ffffff;
  --color-input-border: #d1d5db;
  --color-input-border-hover: #9ca3af;
  --color-input-focus: #2563eb;
  --color-input-placeholder: #9ca3af;
  --color-input-disabled: #f3f4f6;
  --color-ring: #2563eb;
  --color-error: #ef4444;
  --color-error-muted: #fef2f2;
  --color-warning: #f59e0b;
  --color-warning-muted: #fffbeb;
  --color-success: #22c55e;
  --color-success-muted: #f0fdf4;
  --color-info: #3b82f6;
  --color-info-muted: #eff6ff;
}

@layer components {
  .rf-field { @apply mb-4; }
  .rf-label { @apply block mb-1.5 text-sm font-medium text-foreground; }
  .rf-label-required::after { content: " *"; @apply text-error; }

  .rf-input {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 text-sm text-foreground
      placeholder:text-input-placeholder transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed;
  }
  .rf-input-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }

  .rf-textarea {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 text-sm text-foreground
      placeholder:text-input-placeholder transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed resize-y min-h-20;
  }
  .rf-textarea-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }

  .rf-select {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 pr-10 text-sm text-foreground
      transition-colors duration-150 hover:border-input-border-hover focus:outline-none focus:ring-2
      focus:ring-ring/40 focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed
      appearance-none bg-no-repeat;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2364748b' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
    background-position: right 0.75rem center;
    background-size: 1rem;
  }
  .rf-select-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }
  .rf-select-placeholder { @apply text-input-placeholder; }

  .rf-checkbox-wrapper { @apply flex items-start gap-2; }
  .rf-checkbox {
    @apply mt-0.5 h-4 w-4 shrink-0 rounded border border-input-border bg-input
      transition-colors duration-150 hover:border-input-border-hover focus:outline-none
      focus:ring-2 focus:ring-ring/40 focus:ring-offset-0 disabled:opacity-50
      disabled:cursor-not-allowed;
    accent-color: var(--color-primary);
  }
  .rf-checkbox-error { @apply border-error; }
  .rf-checkbox-label { @apply text-sm text-foreground select-none; }

  .rf-radio-group { @apply flex flex-col gap-2; }
  .rf-radio-wrapper { @apply flex items-center gap-2; }
  .rf-radio {
    @apply h-4 w-4 shrink-0 border border-input-border bg-input transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:ring-offset-0 disabled:opacity-50 disabled:cursor-not-allowed;
    accent-color: var(--color-primary);
  }
  .rf-radio-error { @apply border-error; }
  .rf-radio-label { @apply text-sm text-foreground select-none; }

  .rf-error-message { @apply mt-1 text-xs text-error; }
  .rf-note { @apply mt-1 text-xs text-muted; }
}
"#;

pub const FRONTEND_SRC_ADMIN_MAIN_TSX: &str = r#"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "../shared/i18n";
import App from "./App";
import "./app.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter basename="/admin">
      <App />
    </BrowserRouter>
  </StrictMode>,
);
"#;

pub const FRONTEND_SRC_ADMIN_APP_TSX: &str = r#"import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "../shared/ProtectedRoute";
import { useAuthStore } from "./stores/auth";

function DashboardPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold tracking-tight">Rustforge Starter</h1>
        <p className="mt-2 text-lg text-muted">Admin Portal</p>
      </div>
    </div>
  );
}

function LoginPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold tracking-tight">Admin Login</h1>
        <p className="mt-2 text-lg text-muted">Build your login form here.</p>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route path="/*" element={<DashboardPage />} />
      </Route>
    </Routes>
  );
}
"#;

pub const FRONTEND_SRC_ADMIN_APP_CSS: &str = r#"@import "tailwindcss";

@theme {
  --color-background: #0f172a;
  --color-foreground: #f1f5f9;
  --color-muted: #94a3b8;
  --color-muted-foreground: #64748b;
  --color-surface: #1e293b;
  --color-surface-hover: #334155;
  --color-surface-active: #475569;
  --color-primary: #8b5cf6;
  --color-primary-hover: #7c3aed;
  --color-primary-foreground: #ffffff;
  --color-border: #334155;
  --color-border-hover: #475569;
  --color-input: #1e293b;
  --color-input-border: #334155;
  --color-input-border-hover: #475569;
  --color-input-focus: #8b5cf6;
  --color-input-placeholder: #64748b;
  --color-input-disabled: #1a2332;
  --color-ring: #8b5cf6;
  --color-error: #ef4444;
  --color-error-muted: #7f1d1d;
  --color-warning: #f59e0b;
  --color-warning-muted: #78350f;
  --color-success: #22c55e;
  --color-success-muted: #14532d;
  --color-info: #3b82f6;
  --color-info-muted: #1e3a5f;
}

@layer components {
  .rf-field { @apply mb-4; }
  .rf-label { @apply block mb-1.5 text-sm font-medium text-foreground; }
  .rf-label-required::after { content: " *"; @apply text-error; }

  .rf-input {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 text-sm text-foreground
      placeholder:text-input-placeholder transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed;
  }
  .rf-input-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }

  .rf-textarea {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 text-sm text-foreground
      placeholder:text-input-placeholder transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed resize-y min-h-20;
  }
  .rf-textarea-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }

  .rf-select {
    @apply w-full rounded-lg border border-input-border bg-input px-3 py-2 pr-10 text-sm text-foreground
      transition-colors duration-150 hover:border-input-border-hover focus:outline-none focus:ring-2
      focus:ring-ring/40 focus:border-input-focus disabled:opacity-50 disabled:cursor-not-allowed
      appearance-none bg-no-repeat;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
    background-position: right 0.75rem center;
    background-size: 1rem;
  }
  .rf-select-error {
    @apply border-error hover:border-error focus:ring-error/40 focus:border-error;
  }
  .rf-select-placeholder { @apply text-input-placeholder; }

  .rf-checkbox-wrapper { @apply flex items-start gap-2; }
  .rf-checkbox {
    @apply mt-0.5 h-4 w-4 shrink-0 rounded border border-input-border bg-input
      transition-colors duration-150 hover:border-input-border-hover focus:outline-none
      focus:ring-2 focus:ring-ring/40 focus:ring-offset-0 disabled:opacity-50
      disabled:cursor-not-allowed;
    accent-color: var(--color-primary);
  }
  .rf-checkbox-error { @apply border-error; }
  .rf-checkbox-label { @apply text-sm text-foreground select-none; }

  .rf-radio-group { @apply flex flex-col gap-2; }
  .rf-radio-wrapper { @apply flex items-center gap-2; }
  .rf-radio {
    @apply h-4 w-4 shrink-0 border border-input-border bg-input transition-colors duration-150
      hover:border-input-border-hover focus:outline-none focus:ring-2 focus:ring-ring/40
      focus:ring-offset-0 disabled:opacity-50 disabled:cursor-not-allowed;
    accent-color: var(--color-primary);
  }
  .rf-radio-error { @apply border-error; }
  .rf-radio-label { @apply text-sm text-foreground select-none; }

  .rf-error-message { @apply mt-1 text-xs text-error; }
  .rf-note { @apply mt-1 text-xs text-muted; }
}
"#;

pub const FRONTEND_SRC_SHARED_GITKEEP: &str = "";

pub const FRONTEND_SRC_SHARED_I18N_TS: &str = r#"import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "../../../i18n/en.json";
import zh from "../../../i18n/zh.json";

/**
 * Transform Rust-style `:param` placeholders to i18next `{{param}}` syntax.
 * This lets both Rust and React share the same i18n JSON files.
 */
function transformParams(
  obj: Record<string, string>,
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(obj)) {
    result[key] = value.replace(/:([a-zA-Z_]+)/g, "{{$1}}");
  }
  return result;
}

i18n.use(initReactI18next).init({
  fallbackLng: "en",
  keySeparator: false,
  nsSeparator: false,
  interpolation: { escapeValue: false },
  resources: {
    en: { translation: transformParams(en) },
    zh: { translation: transformParams(zh) },
  },
});

export default i18n;
"#;

pub const FRONTEND_SRC_SHARED_CREATE_API_CLIENT_TS: &str = r#"import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from "axios";

export interface ApiClientConfig {
  /** Read the current access token (from auth store). */
  getToken: () => string | null;
  /** Attempt to refresh the session. Must throw on failure. */
  refreshAuth: () => Promise<void>;
  /** Called when refresh also fails — clear state and redirect. */
  onAuthFailure: () => void;
}

/**
 * Factory that creates an Axios instance with:
 * - Request interceptor: attaches `Authorization: Bearer <token>`
 * - Response interceptor: on 401, attempts a single token refresh then
 *   retries the original request. Concurrent 401s share one refresh call.
 */
export function createApiClient(config: ApiClientConfig): AxiosInstance {
  const api = axios.create();

  // ── Request: attach bearer token ────────────────────────
  api.interceptors.request.use((req) => {
    const token = config.getToken();
    if (token) {
      req.headers.Authorization = `Bearer ${token}`;
    }
    return req;
  });

  // ── Response: handle 401 → refresh → retry ─────────────
  let refreshPromise: Promise<void> | null = null;

  api.interceptors.response.use(
    (res) => res,
    async (error) => {
      const original = error.config as InternalAxiosRequestConfig & {
        _retry?: boolean;
      };

      if (error.response?.status !== 401 || original._retry) {
        return Promise.reject(error);
      }

      original._retry = true;

      // Deduplicate concurrent refresh calls
      if (!refreshPromise) {
        refreshPromise = config
          .refreshAuth()
          .finally(() => {
            refreshPromise = null;
          });
      }

      try {
        await refreshPromise;
      } catch {
        config.onAuthFailure();
        return Promise.reject(error);
      }

      // Retry with the new token
      const newToken = config.getToken();
      if (!newToken) {
        config.onAuthFailure();
        return Promise.reject(error);
      }

      original.headers.Authorization = `Bearer ${newToken}`;
      return api(original);
    },
  );

  return api;
}
"#;

pub const FRONTEND_SRC_SHARED_CREATE_AUTH_STORE_TS: &str = r#"import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface Account {
  id: number;
  name: string;
  email: string | null;
}

export interface AuthState<A extends Account = Account> {
  account: A | null;
  token: string | null;
  isLoading: boolean;
  isInitialized: boolean;
  error: string | null;
  login: (credentials: Record<string, unknown>) => Promise<void>;
  logout: () => void;
  fetchAccount: () => Promise<void>;
  refreshToken: () => Promise<void>;
  initSession: () => Promise<void>;
}

export interface AuthConfig {
  loginEndpoint: string;    // "/api/v1/admin/auth/login"
  meEndpoint: string;       // "/api/v1/admin/auth/me"
  refreshEndpoint: string;  // "/api/v1/admin/auth/refresh"
  storageKey: string;       // "admin-auth"
}

/**
 * Factory that creates a typed auth store for any portal.
 *
 * The store uses `client_type: "web"` so the Rust backend stores the
 * refresh token in an HttpOnly cookie. The frontend only manages the
 * access token — the browser sends the cookie automatically on refresh.
 *
 * Usage:
 * ```ts
 * export const useAuthStore = createAuthStore({
 *   loginEndpoint:   "/api/v1/admin/auth/login",
 *   meEndpoint:      "/api/v1/admin/auth/me",
 *   refreshEndpoint: "/api/v1/admin/auth/refresh",
 *   storageKey:      "admin-auth",
 * });
 * ```
 *
 * For portals with extra account fields, pass a generic:
 * ```ts
 * interface MerchantAccount extends Account { companyId: number }
 * export const useAuthStore = createAuthStore<MerchantAccount>({ ... });
 * ```
 */
export function createAuthStore<A extends Account = Account>(
  config: AuthConfig,
) {
  return create<AuthState<A>>()(
    persist(
      (set, get) => ({
        account: null,
        token: null,
        isLoading: false,
        isInitialized: false,
        error: null,

        login: async (credentials: Record<string, unknown>) => {
          set({ isLoading: true, error: null } as Partial<AuthState<A>>);
          try {
            const res = await fetch(config.loginEndpoint, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              credentials: "include",
              body: JSON.stringify({ ...credentials, client_type: "web" }),
            });
            if (!res.ok) {
              const body = await res.json().catch(() => null);
              throw new Error(body?.message ?? "Login failed");
            }
            const { data } = await res.json();
            set({
              token: data.access_token,
              isLoading: false,
            } as Partial<AuthState<A>>);
          } catch (err) {
            set({
              error: (err as Error).message,
              isLoading: false,
            } as Partial<AuthState<A>>);
            throw err;
          }
        },

        logout: () =>
          set({ account: null, token: null } as Partial<AuthState<A>>),

        fetchAccount: async () => {
          const { token } = get();
          if (!token) return;
          set({ isLoading: true } as Partial<AuthState<A>>);
          try {
            const res = await fetch(config.meEndpoint, {
              headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error("Failed to fetch account");
            const { data } = await res.json();
            set({ account: data, isLoading: false } as Partial<AuthState<A>>);
          } catch {
            set({
              account: null,
              token: null,
              isLoading: false,
            } as Partial<AuthState<A>>);
          }
        },

        refreshToken: async () => {
          const res = await fetch(config.refreshEndpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            credentials: "include",
            body: JSON.stringify({ client_type: "web" }),
          });
          if (!res.ok) {
            set({ account: null, token: null } as Partial<AuthState<A>>);
            throw new Error("Session expired");
          }
          const { data } = await res.json();
          set({ token: data.access_token } as Partial<AuthState<A>>);
        },

        initSession: async () => {
          const { token, isInitialized } = get();
          if (isInitialized) return;
          if (!token) {
            set({ isInitialized: true } as Partial<AuthState<A>>);
            return;
          }
          try {
            await get().fetchAccount();
          } catch {
            // Access token expired — try refresh
            try {
              await get().refreshToken();
              await get().fetchAccount();
            } catch {
              // Refresh also failed — session is gone
              set({ account: null, token: null } as Partial<AuthState<A>>);
            }
          }
          set({ isInitialized: true } as Partial<AuthState<A>>);
        },
      }),
      { name: config.storageKey },
    ),
  );
}
"#;

pub const FRONTEND_SRC_SHARED_PROTECTED_ROUTE_TSX: &str = r#"import { useEffect } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import type { AuthState, Account } from "./createAuthStore";
import type { StoreApi, UseBoundStore } from "zustand";

interface Props {
  useAuthStore: UseBoundStore<StoreApi<AuthState<Account>>>;
  loginPath?: string;
}

/**
 * Route guard that protects child routes behind authentication.
 *
 * On first render it calls `initSession()` which:
 * 1. If a persisted token exists → validates it via `fetchAccount()`
 * 2. If access token expired → attempts `refreshToken()` then retries
 * 3. If everything fails → clears auth state
 *
 * While initializing, a loading indicator is shown.
 * Once initialized, unauthenticated users are redirected to `loginPath`.
 *
 * Usage in App.tsx:
 * ```tsx
 * <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
 *   <Route path="/*" element={<DashboardPage />} />
 * </Route>
 * ```
 */
export function ProtectedRoute({ useAuthStore, loginPath = "/login" }: Props) {
  const token = useAuthStore((s) => s.token);
  const isInitialized = useAuthStore((s) => s.isInitialized);
  const initSession = useAuthStore((s) => s.initSession);
  const location = useLocation();

  useEffect(() => {
    initSession();
  }, [initSession]);

  if (!isInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
        <div className="text-muted">Loading…</div>
      </div>
    );
  }

  if (!token) {
    return <Navigate to={loginPath} state={{ from: location }} replace />;
  }

  return <Outlet />;
}
"#;

pub const FRONTEND_SRC_SHARED_COMPONENTS_INDEX_TS: &str = r#"export { TextInput } from "./TextInput";
export type { TextInputProps } from "./TextInput";

export { TextArea } from "./TextArea";
export type { TextAreaProps } from "./TextArea";

export { Select } from "./Select";
export type { SelectProps, SelectOption } from "./Select";

export { Checkbox } from "./Checkbox";
export type { CheckboxProps } from "./Checkbox";

export { Radio } from "./Radio";
export type { RadioProps, RadioOption } from "./Radio";
"#;

pub const FRONTEND_SRC_SHARED_COMPONENTS_TEXT_INPUT_TSX: &str = r##"import { forwardRef, useId, useState, type InputHTMLAttributes } from "react";

type InputType = "text" | "email" | "password" | "search" | "url" | "tel" | "number" | "money" | "pin";

export interface TextInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  type?: InputType;
  label?: string;
  error?: string;
  notes?: string;
}

function formatMoney(value: string): string {
  const num = value.replace(/[^0-9.]/g, "");
  const parts = num.split(".");
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  if (parts.length > 2) parts.length = 2;
  if (parts[1] !== undefined) parts[1] = parts[1].slice(0, 2);
  return parts.join(".");
}

function rawMoney(display: string): string {
  return display.replace(/,/g, "");
}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  ({ type = "text", label, error, notes, required, className, onChange, value, defaultValue, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const isMoney = type === "money";
    const isPin = type === "pin";

    const [moneyDisplay, setMoneyDisplay] = useState(() => {
      const init = (value ?? defaultValue ?? "") as string;
      return isMoney ? formatMoney(init) : "";
    });

    const resolvedType = isMoney ? "text" : isPin ? "password" : type;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (isMoney) {
        const formatted = formatMoney(e.target.value);
        setMoneyDisplay(formatted);
        const synth = { ...e, target: { ...e.target, value: rawMoney(formatted) } } as React.ChangeEvent<HTMLInputElement>;
        onChange?.(synth);
      } else if (isPin) {
        e.target.value = e.target.value.replace(/\D/g, "");
        onChange?.(e);
      } else {
        onChange?.(e);
      }
    };

    const inputMode = isMoney ? "decimal" as const : isPin ? "numeric" as const : undefined;

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={id}
          type={resolvedType}
          inputMode={inputMode}
          required={required}
          className={`rf-input ${error ? "rf-input-error" : ""} ${className ?? ""}`}
          onChange={handleChange}
          value={isMoney ? moneyDisplay : value}
          defaultValue={isMoney ? undefined : defaultValue}
          {...rest}
        />
        {error && <p className="rf-error-message">{error}</p>}
        {notes && !error && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

TextInput.displayName = "TextInput";
"##;

pub const FRONTEND_SRC_SHARED_COMPONENTS_TEXT_AREA_TSX: &str = r##"import { forwardRef, useId, type TextareaHTMLAttributes } from "react";

export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  notes?: string;
}

export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  ({ label, error, notes, required, className, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={id}
          required={required}
          className={`rf-textarea ${error ? "rf-textarea-error" : ""} ${className ?? ""}`}
          {...rest}
        />
        {error && <p className="rf-error-message">{error}</p>}
        {notes && !error && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

TextArea.displayName = "TextArea";
"##;

pub const FRONTEND_SRC_SHARED_COMPONENTS_SELECT_TSX: &str = r##"import { forwardRef, useId, type SelectHTMLAttributes } from "react";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, "children"> {
  options: SelectOption[];
  label?: string;
  error?: string;
  notes?: string;
  placeholder?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ options, label, error, notes, required, placeholder, className, value, defaultValue, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const isPlaceholder = value === "" || (value === undefined && defaultValue === undefined);

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <select
          ref={ref}
          id={id}
          required={required}
          value={value}
          defaultValue={defaultValue}
          className={`rf-select ${error ? "rf-select-error" : ""} ${isPlaceholder ? "rf-select-placeholder" : ""} ${className ?? ""}`}
          {...rest}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} disabled={opt.disabled}>
              {opt.label}
            </option>
          ))}
        </select>
        {error && <p className="rf-error-message">{error}</p>}
        {notes && !error && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

Select.displayName = "Select";
"##;

pub const FRONTEND_SRC_SHARED_COMPONENTS_CHECKBOX_TSX: &str = r##"import { forwardRef, useId, type InputHTMLAttributes } from "react";

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label?: string;
  error?: string;
  notes?: string;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  ({ label, error, notes, className, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;

    return (
      <div className="rf-field">
        <div className="rf-checkbox-wrapper">
          <input
            ref={ref}
            id={id}
            type="checkbox"
            className={`rf-checkbox ${error ? "rf-checkbox-error" : ""} ${className ?? ""}`}
            {...rest}
          />
          {label && (
            <label htmlFor={id} className="rf-checkbox-label">
              {label}
            </label>
          )}
        </div>
        {error && <p className="rf-error-message">{error}</p>}
        {notes && !error && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

Checkbox.displayName = "Checkbox";
"##;

pub const FRONTEND_SRC_SHARED_COMPONENTS_RADIO_TSX: &str = r##"import { useId } from "react";

export interface RadioOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface RadioProps {
  name: string;
  options: RadioOption[];
  value?: string;
  onChange?: (value: string) => void;
  label?: string;
  error?: string;
  notes?: string;
  required?: boolean;
  disabled?: boolean;
  className?: string;
}

export function Radio({ name, options, value, onChange, label, error, notes, required, disabled, className }: RadioProps) {
  const groupId = useId();

  return (
    <div className="rf-field">
      {label && (
        <span className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </span>
      )}
      <div role="radiogroup" aria-labelledby={label ? `${groupId}-label` : undefined} className={`rf-radio-group ${className ?? ""}`}>
        {options.map((opt) => {
          const optId = `${groupId}-${opt.value}`;
          return (
            <div key={opt.value} className="rf-radio-wrapper">
              <input
                id={optId}
                type="radio"
                name={name}
                value={opt.value}
                checked={value === opt.value}
                onChange={() => onChange?.(opt.value)}
                disabled={disabled || opt.disabled}
                className={`rf-radio ${error ? "rf-radio-error" : ""}`}
              />
              <label htmlFor={optId} className="rf-radio-label">
                {opt.label}
              </label>
            </div>
          );
        })}
      </div>
      {error && <p className="rf-error-message">{error}</p>}
      {notes && !error && <p className="rf-note">{notes}</p>}
    </div>
  );
}
"##;

pub const FRONTEND_SRC_USER_STORES_AUTH_TS: &str = r#"import { createAuthStore } from "../../shared/createAuthStore";

export const useAuthStore = createAuthStore({
  loginEndpoint: "/api/v1/auth/login",
  meEndpoint: "/api/v1/auth/me",
  refreshEndpoint: "/api/v1/auth/refresh",
  storageKey: "user-auth",
});
"#;

pub const FRONTEND_SRC_ADMIN_STORES_AUTH_TS: &str = r#"import { createAuthStore } from "../../shared/createAuthStore";
import type { AdminMeOutput } from "../types/admin-auth";

export const useAuthStore = createAuthStore<AdminMeOutput>({
  loginEndpoint: "/api/v1/admin/auth/login",
  meEndpoint: "/api/v1/admin/auth/me",
  refreshEndpoint: "/api/v1/admin/auth/refresh",
  storageKey: "admin-auth",
});
"#;

pub const FRONTEND_SRC_USER_API_TS: &str = r#"import { createApiClient } from "../../shared/createApiClient";
import { useAuthStore } from "./stores/auth";

export const api = createApiClient({
  getToken: () => useAuthStore.getState().token,
  refreshAuth: () => useAuthStore.getState().refreshToken(),
  onAuthFailure: () => {
    useAuthStore.getState().logout();
    window.location.href = "/login";
  },
});
"#;

pub const FRONTEND_SRC_ADMIN_API_TS: &str = r#"import { createApiClient } from "../../shared/createApiClient";
import { useAuthStore } from "./stores/auth";

export const api = createApiClient({
  getToken: () => useAuthStore.getState().token,
  refreshAuth: () => useAuthStore.getState().refreshToken(),
  onAuthFailure: () => {
    useAuthStore.getState().logout();
    window.location.href = "/admin/login";
  },
});
"#;

pub const FRONTEND_AGENTS_MD: &str = r#"# Frontend — Multi-Portal React + Vite + Tailwind 4

This directory contains the frontend source for the Rustforge starter. It ships two independent SPA portals:

| Portal | Base | Dev port | Build output |
|--------|------|----------|--------------|
| **user** | `/` | 5173 | `../public/` (root) |
| **admin** | `/admin/` | 5174 | `../public/admin/` |

Each portal has its own Vite config, HTML entry, CSS theme, and source tree.

## Directory Structure

```
frontend/
├── package.json
├── postcss.config.js
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.user.ts
├── vite.config.admin.ts
├── user.html
├── admin.html
└── src/
    ├── shared/                        # Cross-portal code
    │   ├── types/                     # Generated shared TS types (make gen-types)
    │   │   ├── api.ts                 # ApiResponse<T>, ApiErrorResponse
    │   │   ├── datatable.ts           # DataTable request/response generics
    │   │   └── index.ts               # Barrel export
    │   ├── i18n.ts                    # i18next init (shared JSON, :param transform)
    │   ├── createAuthStore.ts         # Zustand auth store factory
    │   ├── createApiClient.ts         # Axios factory with interceptors
    │   ├── ProtectedRoute.tsx         # Auth guard (route protection + session restore)
    │   └── components/                # Shared form components (styled via rf-* classes)
    │       ├── index.ts               # Barrel export
    │       ├── TextInput.tsx           # text, email, password, search, url, tel, number, money, pin
    │       ├── TextArea.tsx            # Multi-line text
    │       ├── Select.tsx              # Dropdown with typed options
    │       ├── Checkbox.tsx            # Single checkbox
    │       └── Radio.tsx               # Radio group with typed options
    ├── user/
    │   ├── main.tsx                   # Entry (BrowserRouter)
    │   ├── App.tsx                    # Routes
    │   ├── app.css                    # Tailwind 4 theme
    │   ├── api.ts                     # Axios instance for this portal
    │   ├── stores/auth.ts             # Auth store instance
    │   └── types/                     # Generated user TS types (make gen-types)
    │       └── index.ts               # Barrel export (expand as user contracts are added)
    └── admin/
        ├── main.tsx                   # Entry (BrowserRouter basename="/admin")
        ├── App.tsx                    # Routes
        ├── app.css                    # Tailwind 4 theme
        ├── api.ts                     # Axios instance for this portal
        ├── stores/auth.ts             # Auth store instance
        └── types/                     # Generated admin TS types (make gen-types)
            ├── enums.ts               # AdminType, Permission, AuthClientType
            ├── admin.ts               # CRUD: CreateAdminInput, AdminOutput, etc.
            ├── admin-auth.ts          # Auth: AdminLoginInput, AdminMeOutput, etc.
            ├── datatable-admin.ts     # AdminDatatableQueryInput, etc.
            └── index.ts               # Barrel export
```

## Commands

```bash
make dev              # All: Vite user + Vite admin + Rust API
make dev-user         # Vite user portal only (port 5173)
make dev-admin        # Vite admin portal only (port 5174)
make dev-api          # Rust API only (cargo-watch, port 3000)
make build-frontend   # Clean build all portals → public/
```

## Routing (React Router)

Each portal uses `BrowserRouter` from `react-router-dom`. The admin portal sets `basename="/admin"` so all routes are relative to `/admin/`.

Use `<Link to="/login">` and `useNavigate()` — the basename is applied automatically.

### Protected Routes (Auth Guard)

`ProtectedRoute` in `shared/ProtectedRoute.tsx` is the auth middleware. Wrap any routes that require authentication:

```tsx
import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "../shared/ProtectedRoute";
import { useAuthStore } from "./stores/auth";

export default function App() {
  return (
    <Routes>
      {/* Public routes */}
      <Route path="/login" element={<LoginPage />} />

      {/* Protected routes — redirect to /login if unauthenticated */}
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route path="/*" element={<DashboardPage />} />
      </Route>
    </Routes>
  );
}
```

What `ProtectedRoute` does on mount:
1. Calls `initSession()` — checks if a persisted token exists
2. If token exists → calls `fetchAccount()` to validate it
3. If access token expired → calls `refreshToken()` (browser sends HttpOnly cookie) → retries `fetchAccount()`
4. If refresh also fails → clears auth state
5. Shows a loading screen while initializing
6. Once initialized, redirects to `/login` if no valid token, otherwise renders child routes via `<Outlet />`

The `from` location is passed in the redirect state, so after login you can navigate back:

```tsx
const location = useLocation();
const from = location.state?.from?.pathname || "/";
// After successful login:
navigate(from, { replace: true });
```

### Custom login path

Pass `loginPath` prop if the portal uses a different login route:

```tsx
<Route element={<ProtectedRoute useAuthStore={useAuthStore} loginPath="/auth/signin" />}>
```

## API Client (Axios)

Each portal has its own `api.ts` that exports a configured Axios instance. The shared factory (`createApiClient`) provides:

- **Request interceptor**: attaches `Authorization: Bearer <token>` from the auth store
- **Response interceptor**: on 401, attempts token refresh (one concurrent refresh), retries the request, or redirects to login on failure

```tsx
// Import the portal's api instance for all API calls
import { api } from "./api";

const res = await api.get("/api/v1/articles");
const data = res.data;
```

The refresh uses `client_type: "web"` — the Rust backend stores the refresh token in an HttpOnly cookie. The frontend only manages the access token; the browser sends the cookie automatically.

### Auth Flow

1. **Login**: `useAuthStore.login({ username, password })` → POST with `client_type: "web"` → stores `access_token`, refresh token set as HttpOnly cookie by server
2. **Page refresh**: `ProtectedRoute` calls `initSession()` → validates persisted token → refreshes if expired → loads account data
3. **API calls**: Axios attaches bearer token automatically
4. **401 response**: interceptor calls `refreshToken()` → POST to refresh endpoint (cookie sent automatically) → new `access_token` → retry original request
5. **Refresh failure**: clears auth state, redirects to `/login`

## i18n (Shared with Rust)

Frontend and Rust share the same `i18n/*.json` files. The Rust backend uses `:param` syntax; `src/shared/i18n.ts` transforms `:param` → `{{param}}` at init time so i18next can interpolate.

```tsx
import { useTranslation } from "react-i18next";

function Greeting({ name }: { name: string }) {
  const { t } = useTranslation();
  return <p>{t("Welcome :name", { name })}</p>;
}
```

The key is the English text itself — if no translation is found, the key is the fallback.

## TypeScript Types (Generated)

Type definitions in `*/types/` directories are **auto-generated** from Rust contract structs using `ts-rs`. Do not edit them manually — run `make gen-types` to regenerate after changing Rust contracts.

### Usage

```typescript
import type { ApiResponse } from "../../shared/types";
import type { AdminOutput, CreateAdminInput } from "../types";

// Typed API calls
const res = await api.post<ApiResponse<AdminOutput>>("/api/v1/admin", input);
const admin: AdminOutput = res.data.data;
```

### Regeneration

```bash
make gen-types    # Regenerate frontend TS types from Rust contracts
make gen          # Code generation + type generation
```

### How it works

1. Rust contract structs derive `ts_rs::TS` with `#[ts(export, export_to = "admin/types/")]`
2. `app/src/bin/export-types.rs` calls `T::export_to_string()` for each contract type
3. The binary assembles complete `.ts` files with proper imports and writes to `frontend/src/`
4. Framework types (ApiResponse, DataTable*) and enum types are emitted as static strings

### Adding types for a new domain

1. In your Rust contract, add `#[derive(TS)]` and `#[ts(export, export_to = "{portal}/types/")]`
2. For fields using external types (generated enums, framework types), add `#[ts(type = "TypeName")]`
3. Register the types in `app/src/bin/export-types.rs` (add a new `TsFile` block)
4. Update the barrel `index.ts` to re-export the new module
5. Run `make gen-types`

### Type mapping conventions

| Rust | TypeScript | Notes |
|------|-----------|-------|
| `String` | `string` | |
| `i64`, `f64` | `number` | |
| `bool` | `boolean` | |
| `Option<T>` | `T \| null` | |
| `Vec<T>` | `T[]` | |
| `time::OffsetDateTime` | `string` | Use `#[ts(type = "string")]` |
| `UsernameString` (newtype) | `string` | Use `#[ts(type = "string")]` |
| `AdminType` (generated enum) | `AdminType` | Use `#[ts(type = "AdminType")]` |
| `#[serde(skip)]` field | omitted | ts-rs respects serde attrs |

## State Management (Zustand)

Use Zustand for state. Define stores in `src/{portal}/stores/`.

### Auth Store Factory

`src/shared/createAuthStore.ts` is a factory that creates a typed auth store for any portal. Each portal provides its own endpoints:

```typescript
// src/{portal}/stores/auth.ts
import { createAuthStore } from "../../shared/createAuthStore";

export const useAuthStore = createAuthStore({
  loginEndpoint:   "/api/v1/{portal}/auth/login",
  meEndpoint:      "/api/v1/{portal}/auth/me",
  refreshEndpoint: "/api/v1/{portal}/auth/refresh",
  storageKey:      "{portal}-auth",
});
```

The `login` action accepts a generic credentials object — each portal passes whatever fields its API expects:

```tsx
// Admin login (uses username)
await login({ username, password });

// User login (might use email)
await login({ email, password });
```

`client_type: "web"` is appended automatically.

For portals with extra account fields, pass a generic:

```typescript
import { createAuthStore, type Account } from "../../shared/createAuthStore";

interface MerchantAccount extends Account {
  companyId: number;
  companyName: string;
}

export const useAuthStore = createAuthStore<MerchantAccount>({
  loginEndpoint:   "/api/v1/merchant/auth/login",
  meEndpoint:      "/api/v1/merchant/auth/me",
  refreshEndpoint: "/api/v1/merchant/auth/refresh",
  storageKey:      "merchant-auth",
});
```

### Creating Other Shared Store Factories

Follow the same factory pattern as `createAuthStore` for any cross-portal store. Define the factory in `shared/`, instantiate with portal-specific config in `src/{portal}/stores/`.

## Tailwind CSS 4

Each portal customises its design tokens in its own `app.css` via `@theme { }`. No `tailwind.config.js` is used — Tailwind 4 reads theme configuration from CSS.

```css
@import "tailwindcss";

@theme {
  --color-primary: #2563eb;
}
```

### Theme Tokens

Each portal defines a comprehensive set of semantic color tokens in `@theme`. The admin portal uses a dark scheme and the user portal uses a light scheme. Key token groups:

| Group | Tokens | Purpose |
|-------|--------|---------|
| **Base** | `background`, `foreground`, `muted`, `muted-foreground` | Page background, text, subtle text |
| **Surface** | `surface`, `surface-hover`, `surface-active` | Cards, panels, interactive elements |
| **Primary** | `primary`, `primary-hover`, `primary-foreground` | Brand color, buttons, links |
| **Border** | `border`, `border-hover` | General dividers, card borders |
| **Input** | `input`, `input-border`, `input-border-hover`, `input-focus`, `input-placeholder`, `input-disabled` | Form control styling |
| **Ring** | `ring` | Focus ring color |
| **Status** | `error`/`error-muted`, `warning`/`warning-muted`, `success`/`success-muted`, `info`/`info-muted` | Validation, alerts, badges |

## Shared Form Components

Reusable form components live in `src/shared/components/`. They contain **zero hardcoded Tailwind utilities** — all visual styling is applied through `rf-*` CSS classes defined in each portal's `app.css` using `@layer components` + `@apply`.

This means portals can have completely different visual styles while sharing identical React logic.

### Available Components

| Component | Import | Description |
|-----------|--------|-------------|
| `TextInput` | `TextInputProps` | Text, email, password, search, url, tel, number + special `money` and `pin` types |
| `TextArea` | `TextAreaProps` | Multi-line text input |
| `Select` | `SelectProps`, `SelectOption` | Dropdown with typed options |
| `Checkbox` | `CheckboxProps` | Single checkbox with label |
| `Radio` | `RadioProps`, `RadioOption` | Radio group with typed options |

### Usage

```tsx
import { TextInput, TextArea, Select, Checkbox, Radio } from "../shared/components";

// Basic text input with error
<TextInput label="Email" type="email" required error={errors.email} />

// Money input — displays formatted (1,234.56), onChange emits raw numeric string
<TextInput label="Amount" type="money" onChange={(e) => setAmount(e.target.value)} />

// PIN input — renders as password field, strips non-digits, numeric keyboard
<TextInput label="PIN" type="pin" maxLength={6} />

// Text area with helper notes
<TextArea label="Bio" notes="Maximum 500 characters" rows={4} />

// Select with placeholder
<Select
  label="Country"
  placeholder="Choose a country..."
  options={[
    { value: "us", label: "United States" },
    { value: "uk", label: "United Kingdom" },
  ]}
  required
/>

// Checkbox
<Checkbox label="I agree to the terms" error={errors.terms} />

// Radio group
<Radio
  name="role"
  label="Role"
  value={role}
  onChange={setRole}
  options={[
    { value: "admin", label: "Administrator" },
    { value: "editor", label: "Editor" },
    { value: "viewer", label: "Viewer" },
  ]}
/>
```

### Error and Notes Pattern

All components follow the same pattern:
- `error` prop: shows a red error message below the input and applies error styling
- `notes` prop: shows a grey helper note below the input (hidden when `error` is present)
- `required` prop: adds a red asterisk after the label

### Special TextInput Types

- **`money`**: Formats display value with commas (`1,234.56`), emits raw numeric string via `onChange`, uses `inputMode="decimal"` for mobile numeric keyboard
- **`pin`**: Renders as `type="password"`, strips non-digit characters, uses `inputMode="numeric"` for mobile numeric keyboard

### CSS Class Reference

Each portal's `app.css` defines these `rf-*` classes using `@apply` with theme tokens:

| Class | Used by | Purpose |
|-------|---------|---------|
| `rf-field` | All | Wrapper div with bottom margin |
| `rf-label` | All | Label styling |
| `rf-label-required` | All | Adds red asterisk via `::after` |
| `rf-input` / `rf-input-error` | TextInput | Text input styling |
| `rf-textarea` / `rf-textarea-error` | TextArea | Textarea styling |
| `rf-select` / `rf-select-error` / `rf-select-placeholder` | Select | Select dropdown styling |
| `rf-checkbox-wrapper` / `rf-checkbox` / `rf-checkbox-error` / `rf-checkbox-label` | Checkbox | Checkbox layout and styling |
| `rf-radio-group` / `rf-radio-wrapper` / `rf-radio` / `rf-radio-error` / `rf-radio-label` | Radio | Radio group layout and styling |
| `rf-error-message` | All | Error text below input |
| `rf-note` | All | Helper text below input |

### Theming for New Portals

When adding a new portal, copy the `@layer components` block from an existing portal's `app.css`. The visual appearance is controlled entirely by the `@theme` tokens — the same `rf-*` class definitions produce different results based on each portal's token values.

## Adding a New Portal

1. Create `vite.config.{name}.ts` — set `base`, `server.port`, `build.outDir`.
2. Create `{name}.html` entry point.
3. Create `src/{name}/main.tsx` — `BrowserRouter` with `basename="/{name}"`.
4. Create `src/{name}/App.tsx` — routes with `<ProtectedRoute>` wrapping protected routes.
5. Create `src/{name}/app.css` — Tailwind theme.
6. Create `src/{name}/stores/auth.ts` — call `createAuthStore` with portal config.
7. Create `src/{name}/api.ts` — call `createApiClient` wired to auth store.
8. Add `dev:{name}` and `build:{name}` scripts to `package.json`.
9. Update the `build` script ordering (build nested portals first).
10. In Rust, add `nest_service("/{name}", ...)` in `build_router` (see `app/src/internal/api/mod.rs`).

## Production

`make build-frontend` writes optimised assets into `public/`. The Rust API serves them:
- `/admin/*` → `public/admin/index.html` (admin SPA fallback)
- `/*` → `public/index.html` (user SPA fallback)
"#;

// ── TypeScript type files ───────────────────────────────────────

pub const FRONTEND_SRC_SHARED_TYPES_API_TS: &str = r#"export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface ApiErrorResponse {
  message: string;
  errors?: Record<string, string[]>;
}
"#;

pub const FRONTEND_SRC_SHARED_TYPES_DATATABLE_TS: &str = r#"export type DataTablePaginationMode = "offset" | "cursor";

export type DataTableSortDirection = "asc" | "desc";

export interface DataTableQueryRequestBase {
  include_meta?: boolean;
  page?: number | null;
  per_page?: number | null;
  cursor?: string | null;
  pagination_mode?: DataTablePaginationMode | null;
  sorting_column?: string | null;
  sorting?: DataTableSortDirection | null;
  timezone?: string | null;
  created_at_from?: string | null;
  created_at_to?: string | null;
}

export interface DataTableEmailExportRequestBase {
  query: DataTableQueryRequestBase;
  recipients: string[];
  subject?: string | null;
  export_file_name?: string | null;
}

export type DataTableFilterFieldType =
  | "text"
  | "select"
  | "number"
  | "date"
  | "datetime"
  | "boolean";

export interface DataTableFilterOptionDto {
  label: string;
  value: string;
}

export interface DataTableFilterFieldDto {
  field: string;
  filter_key: string;
  type: DataTableFilterFieldType;
  label: string;
  placeholder?: string;
  description?: string;
  options?: DataTableFilterOptionDto[];
}

export interface DataTableColumnMetaDto {
  name: string;
  data_type: string;
  sortable: boolean;
  localized: boolean;
  filter_ops: string[];
}

export interface DataTableRelationColumnMetaDto {
  relation: string;
  column: string;
  data_type: string;
  filter_ops: string[];
}

export interface DataTableDefaultsDto {
  sorting_column: string;
  sorted: string;
  per_page: number;
  export_ignore_columns: string[];
  timestamp_columns: string[];
  unsortable: string[];
}

export interface DataTableDiagnosticsDto {
  duration_ms: number;
  auto_filters_applied: number;
  unknown_filters: string[];
  unknown_filter_mode: string;
}

export interface DataTableMetaDto {
  model_key: string;
  defaults: DataTableDefaultsDto;
  columns: DataTableColumnMetaDto[];
  relation_columns: DataTableRelationColumnMetaDto[];
  filter_rows: DataTableFilterFieldDto[][];
}

export interface DataTableQueryResponse<T> {
  records: T[];
  per_page: number;
  total_records: number;
  total_pages: number;
  page: number;
  pagination_mode: string;
  has_more?: boolean;
  next_cursor?: string;
  diagnostics: DataTableDiagnosticsDto;
  meta?: DataTableMetaDto;
}

export type DataTableEmailExportState =
  | "waiting_csv"
  | "uploading"
  | "sending"
  | "completed"
  | "failed";

export interface DataTableEmailExportStatusDto {
  state: DataTableEmailExportState;
  recipients: string[];
  subject?: string;
  link_url?: string;
  error?: string;
  updated_at_unix: number;
  sent_at_unix?: number;
}

export interface DataTableEmailExportQueuedDto {
  job_id: string;
  csv_state: string;
  email_state: DataTableEmailExportState;
}

export interface DataTableExportStatusResponseDto {
  job_id: string;
  model_key: string;
  csv_state: string;
  csv_error?: string;
  csv_file_name?: string;
  csv_content_type?: string;
  csv_total_records?: number;
  email?: DataTableEmailExportStatusDto;
}
"#;

pub const FRONTEND_SRC_SHARED_TYPES_INDEX_TS: &str = r#"export * from "./api";
export * from "./datatable";
"#;

pub const FRONTEND_SRC_ADMIN_TYPES_ENUMS_TS: &str = r#"export type AdminType = "Developer" | "SuperAdmin" | "Admin";

export type Permission = "admin.read" | "admin.manage";

export type AuthClientType = "web" | "mobile";
"#;

pub const FRONTEND_SRC_ADMIN_TYPES_ADMIN_TS: &str = r#"import type { AdminType, Permission } from "./enums";

export interface CreateAdminInput {
  username: string;
  email?: string | null;
  name: string;
  admin_type: AdminType;
  password: string;
  abilities?: Permission[];
}

export interface UpdateAdminInput {
  username?: string | null;
  email?: string | null;
  name?: string | null;
  admin_type?: AdminType | null;
  abilities?: Permission[] | null;
}

export interface AdminOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
  abilities: string[];
  created_at: string;
  updated_at: string;
}

export interface AdminDeleteOutput {
  deleted: boolean;
}
"#;

pub const FRONTEND_SRC_ADMIN_TYPES_ADMIN_AUTH_TS: &str = r#"import type { AdminType, AuthClientType } from "./enums";

export interface AdminLoginInput {
  username: string;
  password: string;
  client_type: AuthClientType;
}

export interface AdminRefreshInput {
  client_type: AuthClientType;
  refresh_token?: string | null;
}

export interface AdminLogoutInput {
  client_type: AuthClientType;
  refresh_token?: string | null;
}

export interface AdminProfileUpdateInput {
  name: string;
  email?: string | null;
}

export interface AdminPasswordUpdateInput {
  current_password: string;
  password: string;
  password_confirmation: string;
}

export interface AdminAuthOutput {
  token_type: string;
  access_token: string;
  access_expires_at?: string | null;
  refresh_token?: string;
  scopes: string[];
}

export interface AdminMeOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
  scopes: string[];
}

export interface AdminProfileUpdateOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
}

export interface AdminPasswordUpdateOutput {
  updated: boolean;
}

export interface AdminLogoutOutput {
  revoked: boolean;
}
"#;

pub const FRONTEND_SRC_ADMIN_TYPES_DATATABLE_ADMIN_TS: &str = r#"import type { AdminType } from "./enums";
import type {
  DataTableQueryRequestBase,
  DataTableEmailExportRequestBase,
} from "../../shared/types/datatable";

export interface AdminDatatableQueryInput {
  base?: DataTableQueryRequestBase;
  q?: string | null;
  username?: string | null;
  email?: string | null;
  admin_type?: AdminType | null;
}

export interface AdminDatatableEmailExportInput {
  base: DataTableEmailExportRequestBase;
  q?: string | null;
  username?: string | null;
  email?: string | null;
  admin_type?: AdminType | null;
}
"#;

pub const FRONTEND_SRC_ADMIN_TYPES_INDEX_TS: &str = r#"export * from "./enums";
export * from "./admin";
export * from "./admin-auth";
export * from "./datatable-admin";
"#;

pub const FRONTEND_SRC_USER_TYPES_INDEX_TS: &str = r#"// Add user-specific types here as user contracts are created.
// Example:
//   export * from "./user";
//   export * from "./user-auth";
"#;

pub const APP_BIN_EXPORT_TYPES_RS: &str = r##"//! Exports Rust contract types to TypeScript.
//!
//! Uses `ts-rs` to convert Rust types with `#[derive(TS)]` into TypeScript
//! definitions, then writes them to `frontend/src/` alongside static framework
//! types (ApiResponse, DataTable*, enums).
//!
//! Run: `cargo run -p app --bin export-types`
//! Or:  `make gen-types`

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use ts_rs::TS;

// ── Generated types (ts-rs) ──────────────────────────────────

/// A generated TypeScript file: imports + ts-rs type definitions.
struct TsFile {
    /// Relative path from `frontend/src/`, e.g. `admin/types/admin.ts`
    rel_path: &'static str,
    /// Import lines prepended to the file.
    imports: &'static [&'static str],
    /// TypeScript definitions produced by ts-rs (collected at runtime).
    definitions: Vec<String>,
}

fn main() {
    let base = Path::new("frontend/src");

    // ── 1. Contract types via ts-rs ─────────────────────────
    let mut files: Vec<TsFile> = Vec::new();

    // admin/types/admin.ts
    {
        use app::contracts::api::v1::admin::*;
        files.push(TsFile {
            rel_path: "admin/types/admin.ts",
            imports: &[r#"import type { AdminType, Permission } from "./enums";"#],
            definitions: vec![
                CreateAdminInput::export_to_string().expect("CreateAdminInput"),
                UpdateAdminInput::export_to_string().expect("UpdateAdminInput"),
                AdminOutput::export_to_string().expect("AdminOutput"),
                AdminDeleteOutput::export_to_string().expect("AdminDeleteOutput"),
            ],
        });
    }

    // admin/types/admin-auth.ts
    {
        use app::contracts::api::v1::admin_auth::*;
        files.push(TsFile {
            rel_path: "admin/types/admin-auth.ts",
            imports: &[r#"import type { AdminType, AuthClientType } from "./enums";"#],
            definitions: vec![
                AdminLoginInput::export_to_string().expect("AdminLoginInput"),
                AdminRefreshInput::export_to_string().expect("AdminRefreshInput"),
                AdminLogoutInput::export_to_string().expect("AdminLogoutInput"),
                AdminProfileUpdateInput::export_to_string().expect("AdminProfileUpdateInput"),
                AdminPasswordUpdateInput::export_to_string().expect("AdminPasswordUpdateInput"),
                AdminAuthOutput::export_to_string().expect("AdminAuthOutput"),
                AdminMeOutput::export_to_string().expect("AdminMeOutput"),
                AdminProfileUpdateOutput::export_to_string().expect("AdminProfileUpdateOutput"),
                AdminPasswordUpdateOutput::export_to_string().expect("AdminPasswordUpdateOutput"),
                AdminLogoutOutput::export_to_string().expect("AdminLogoutOutput"),
            ],
        });
    }

    // admin/types/datatable-admin.ts
    {
        use app::contracts::datatable::admin::admin::*;
        files.push(TsFile {
            rel_path: "admin/types/datatable-admin.ts",
            imports: &[
                r#"import type { AdminType } from "./enums";"#,
                r#"import type { DataTableQueryRequestBase, DataTableEmailExportRequestBase } from "../../shared/types/datatable";"#,
            ],
            definitions: vec![
                AdminDatatableQueryInput::export_to_string().expect("AdminDatatableQueryInput"),
                AdminDatatableEmailExportInput::export_to_string().expect("AdminDatatableEmailExportInput"),
            ],
        });
    }

    // Write ts-rs generated files
    for ts_file in &files {
        let path = base.join(ts_file.rel_path);
        write_file(&path, &assemble(ts_file));
    }

    // ── 2. Static framework types (not derived from Rust structs) ──
    //
    // These mirror core-web types that don't live in the app crate.
    // The scaffold also writes identical initial copies; this binary
    // overwrites them to keep everything in sync after contract changes.
    let statics: &[(&str, &str)] = &[
        ("shared/types/api.ts", SHARED_API_TS),
        ("shared/types/datatable.ts", SHARED_DATATABLE_TS),
        ("shared/types/index.ts", SHARED_INDEX_TS),
        ("admin/types/enums.ts", ADMIN_ENUMS_TS),
        ("admin/types/index.ts", ADMIN_INDEX_TS),
        ("user/types/index.ts", USER_INDEX_TS),
    ];
    for (rel, content) in statics {
        write_file(&base.join(rel), content);
    }

    println!("\nTypeScript types regenerated in frontend/src/");
}

// ── Helpers ──────────────────────────────────────────────────

fn assemble(f: &TsFile) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);
    for imp in f.imports {
        out.push_str(imp);
        out.push('\n');
    }
    out.push('\n');
    for (i, def) in f.definitions.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(def);
        out.push('\n');
    }
    out
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create directory");
    }
    fs::write(path, content).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", path.display());
    });
    println!("  wrote {}", path.display());
}

// ── Static TypeScript content ────────────────────────────────
// Framework types from core-web that can't derive TS directly.

const SHARED_API_TS: &str = "\
export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface ApiErrorResponse {
  message: string;
  errors?: Record<string, string[]>;
}
";

const SHARED_DATATABLE_TS: &str = "\
export type DataTablePaginationMode = \"offset\" | \"cursor\";

export type DataTableSortDirection = \"asc\" | \"desc\";

export interface DataTableQueryRequestBase {
  include_meta?: boolean;
  page?: number | null;
  per_page?: number | null;
  cursor?: string | null;
  pagination_mode?: DataTablePaginationMode | null;
  sorting_column?: string | null;
  sorting?: DataTableSortDirection | null;
  timezone?: string | null;
  created_at_from?: string | null;
  created_at_to?: string | null;
}

export interface DataTableEmailExportRequestBase {
  query: DataTableQueryRequestBase;
  recipients: string[];
  subject?: string | null;
  export_file_name?: string | null;
}

export type DataTableFilterFieldType =
  | \"text\"
  | \"select\"
  | \"number\"
  | \"date\"
  | \"datetime\"
  | \"boolean\";

export interface DataTableFilterOptionDto {
  label: string;
  value: string;
}

export interface DataTableFilterFieldDto {
  field: string;
  filter_key: string;
  type: DataTableFilterFieldType;
  label: string;
  placeholder?: string;
  description?: string;
  options?: DataTableFilterOptionDto[];
}

export interface DataTableColumnMetaDto {
  name: string;
  data_type: string;
  sortable: boolean;
  localized: boolean;
  filter_ops: string[];
}

export interface DataTableRelationColumnMetaDto {
  relation: string;
  column: string;
  data_type: string;
  filter_ops: string[];
}

export interface DataTableDefaultsDto {
  sorting_column: string;
  sorted: string;
  per_page: number;
  export_ignore_columns: string[];
  timestamp_columns: string[];
  unsortable: string[];
}

export interface DataTableDiagnosticsDto {
  duration_ms: number;
  auto_filters_applied: number;
  unknown_filters: string[];
  unknown_filter_mode: string;
}

export interface DataTableMetaDto {
  model_key: string;
  defaults: DataTableDefaultsDto;
  columns: DataTableColumnMetaDto[];
  relation_columns: DataTableRelationColumnMetaDto[];
  filter_rows: DataTableFilterFieldDto[][];
}

export interface DataTableQueryResponse<T> {
  records: T[];
  per_page: number;
  total_records: number;
  total_pages: number;
  page: number;
  pagination_mode: string;
  has_more?: boolean;
  next_cursor?: string;
  diagnostics: DataTableDiagnosticsDto;
  meta?: DataTableMetaDto;
}

export type DataTableEmailExportState =
  | \"waiting_csv\"
  | \"uploading\"
  | \"sending\"
  | \"completed\"
  | \"failed\";

export interface DataTableEmailExportStatusDto {
  state: DataTableEmailExportState;
  recipients: string[];
  subject?: string;
  link_url?: string;
  error?: string;
  updated_at_unix: number;
  sent_at_unix?: number;
}

export interface DataTableEmailExportQueuedDto {
  job_id: string;
  csv_state: string;
  email_state: DataTableEmailExportState;
}

export interface DataTableExportStatusResponseDto {
  job_id: string;
  model_key: string;
  csv_state: string;
  csv_error?: string;
  csv_file_name?: string;
  csv_content_type?: string;
  csv_total_records?: number;
  email?: DataTableEmailExportStatusDto;
}
";

const SHARED_INDEX_TS: &str = "\
export * from \"./api\";
export * from \"./datatable\";
";

const ADMIN_ENUMS_TS: &str = "\
export type AdminType = \"Developer\" | \"SuperAdmin\" | \"Admin\";

export type Permission = \"admin.read\" | \"admin.manage\";

export type AuthClientType = \"web\" | \"mobile\";
";

const ADMIN_INDEX_TS: &str = "\
export * from \"./enums\";
export * from \"./admin\";
export * from \"./admin-auth\";
export * from \"./datatable-admin\";
";

const USER_INDEX_TS: &str = "\
// Add user-specific types here as user contracts are created.
// Example:
//   export * from \"./user\";
//   export * from \"./user-auth\";
";
"##;
