#!/usr/bin/env bash
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
