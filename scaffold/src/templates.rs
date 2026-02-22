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
APP_KEY=dev-only
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
.DS_Store
*.log
*.tmp

# Keep the directory, ignore generated static files by default.
public/*
!public/.gitkeep
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
	@echo "  make dev"
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

.PHONY: install-tools
install-tools:
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch

.PHONY: dev
dev:
	@command -v cargo-watch >/dev/null 2>&1 || (echo "cargo-watch not found. Run: make install-tools" && exit 1)
	RUN_WORKER=true cargo watch -x "run -p app --bin api-server"

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
./console migrate pump
./console migrate run
```

5. Start services:

```bash
./bin/api-server
./bin/websocket-server
./bin/worker
```

## Daily Commands

```bash
make dev
make check
make run-api
make run-ws
make run-worker
./console migrate pump
./console migrate run
make server-install
make server-update
make framework-docs-build
```

## Ubuntu Server Install (Interactive)

Run as root on Ubuntu 24/25:

```bash
sudo ./scripts/install-ubuntu.sh
# or
make server-install
```

The installer is idempotent (safe to run multiple times) and will:
- create/reuse an isolated Linux user per project
- configure SSH access (copy root key, manual key, or generated password)
- recursively `chown` project files to the isolated user
- upsert `.env` values (domain/env/db/redis/ports)
- generate/update nginx site config
- optionally configure Supervisor programs
- optionally issue/renew Let's Encrypt certificates with cron renewal

## Server Update Script

Use the generated update helper for deploy-like updates:

```bash
./scripts/update.sh
# optional opt-out
RUN_MIGRATIONS=false ./scripts/update.sh
```

It will:
- `git pull --ff-only`
- compile release binaries (`cargo build --release --workspace`)
- run migrations by default (set `RUN_MIGRATIONS=false` to skip)
- reread/update and restart Supervisor programs from the installed supervisor config

## i18n Ownership

This starter owns translation files.
`I18N_DIR=i18n` is set in `.env.example`, and API locale is resolved from `Accept-Language`/`x-locale` by framework middleware.

## Static Assets (Optional)

1. Keep `PUBLIC_PATH=public` (or set your own path in `.env`).
2. Build your frontend project (for example Vite `dist` output).
3. Publish files into `PUBLIC_PATH`:

```bash
./console assets publish --from frontend/dist --clean
```

When `PUBLIC_PATH/index.html` exists, API server serves that folder at `/` with SPA fallback.

## Redis Key Isolation

Keep `REDIS_CACHE_PREFIX` empty by default. Framework auto-derives `{APP_NAME}_{APP_ENV}` to namespace keys.
Set `REDIS_CACHE_PREFIX` only when you need a custom prefix strategy.

## Dependency Mode

This starter uses git dependencies to Rustforge.
For production stability, pin to a tag in `Cargo.toml`.

`make framework-docs-build` publishes framework docs assets into
`PUBLIC_PATH + FRAMEWORK_DOCS_PATH` (default: `public/framework-documentation`).
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
"#;

pub const APP_LIB_RS: &str = r#"pub mod contracts;
pub mod internal;
pub mod seeds;
pub mod validation;
"#;

pub const APP_CONTRACTS_MOD_RS: &str = r#"pub mod api;
pub mod datatable;
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
    DataTableFilterOptionDto, DataTableQueryRequestBase, DataTableQueryRequestContract,
    DataTableScopedContract,
};
use generated::models::{AdminType, AdminView};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminDatatableQueryInput {
    #[serde(default)]
    #[validate(nested)]
    pub base: DataTableQueryRequestBase,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub q: Option<String>,
    #[serde(default)]
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub email: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminDatatableEmailExportInput {
    #[validate(nested)]
    pub base: DataTableEmailExportRequestBase,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub q: Option<String>,
    #[serde(default)]
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub email: Option<String>,
    #[serde(default)]
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

pub const APP_CONTRACTS_API_V1_ADMIN_RS: &str = r#"use generated::{models::AdminType, permissions::Permission};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct CreateAdminInput {
    #[validate(custom(function = "crate::validation::username::validate_username"))]
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: String,
    #[serde(default)]
    #[validate(email)]
    #[schemars(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    pub admin_type: AdminType,
    #[validate(length(min = 8, max = 128))]
    #[schemars(length(min = 8, max = 128))]
    pub password: String,
    #[serde(default)]
    pub abilities: Vec<Permission>,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct UpdateAdminInput {
    #[serde(default)]
    #[validate(custom(function = "crate::validation::username::validate_username"))]
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: Option<String>,
    #[serde(default)]
    #[validate(email)]
    #[schemars(email)]
    pub email: Option<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[serde(default)]
    pub admin_type: Option<AdminType>,
    #[serde(default)]
    pub abilities: Option<Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminDeleteOutput {
    pub deleted: bool,
}
"#;

pub const APP_CONTRACTS_API_V1_ADMIN_AUTH_RS: &str = r#"use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;
use core_web::auth::AuthClientType;
use generated::models::AdminType;

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminLoginInput {
    #[validate(custom(function = "crate::validation::username::validate_username"))]
    #[validate(length(min = 3, max = 64))]
    #[schemars(length(min = 3, max = 64))]
    pub username: String,

    #[validate(length(min = 8, max = 128))]
    #[schemars(length(min = 8, max = 128))]
    pub password: String,

    pub client_type: AuthClientType,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminRefreshInput {
    pub client_type: AuthClientType,
    #[serde(default)]
    #[validate(length(min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminLogoutInput {
    pub client_type: AuthClientType,
    #[serde(default)]
    #[validate(length(min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminProfileUpdateInput {
    #[validate(length(min = 1, max = 120))]
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    #[serde(default)]
    #[validate(email)]
    #[schemars(email)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AdminPasswordUpdateInput {
    #[validate(length(min = 8, max = 128))]
    #[schemars(length(min = 8, max = 128))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    #[validate(must_match(other = "password_confirmation"))]
    #[schemars(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 8, max = 128))]
    #[schemars(length(min = 8, max = 128))]
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminAuthOutput {
    pub token_type: String,
    pub access_token: String,
    #[schemars(with = "Option<String>")]
    pub access_expires_at: Option<time::OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminMeOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminProfileUpdateOutput {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub name: String,
    pub admin_type: AdminType,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AdminPasswordUpdateOutput {
    pub updated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
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
    contracts::ContractJson,
    error::AppError,
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
    req: ContractJson<CreateAdminInput>,
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
    let admin = workflow::update(&state, &auth, id, req.0).await?;
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
    models::{Admin, AdminQuery, AdminType, AdminView},
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
    ensure_username_available(&state.db, &username, None).await?;

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
            ensure_username_available(&state.db, &username, Some(id)).await?;
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

async fn ensure_username_available(
    db: &sqlx::PgPool,
    username: &str,
    exclude_id: Option<i64>,
) -> Result<(), AppError> {
    let mut query = AdminQuery::new(DbConn::pool(db), None).where_username(Op::Eq, username.to_string());
    if let Some(id) = exclude_id {
        query = query.where_id(Op::Ne, id);
    }
    let exists = query.first().await.map_err(AppError::from)?.is_some();
    if exists {
        return Err(AppError::BadRequest(t("Username is already taken")));
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
