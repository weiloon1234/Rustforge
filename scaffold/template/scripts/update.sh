#!/usr/bin/env bash
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
