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
    if supervisorctl "$@" 2>/dev/null; then
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

PROJECT_USER="$(read_env_value "${ENV_FILE}" "PROJECT_USER")"
SUPERVISOR_PROJECT_SLUG="$(read_env_value "${ENV_FILE}" "SUPERVISOR_PROJECT_SLUG")"
RUN_MIGRATIONS="${RUN_MIGRATIONS:-true}"

echo "Deploying..."
echo "  Project dir : ${PROJECT_DIR}"
echo "  User        : ${PROJECT_USER:-<current user>}"
echo "  Supervisor  : ${SUPERVISOR_PROJECT_SLUG:-<not configured>}"
echo

# 1. Pull latest code
if [[ -d "${PROJECT_DIR}/.git" ]]; then
    echo "==> git pull"
    run_as_project_user "cd \"${PROJECT_DIR}\" && git pull --ff-only"
fi

# 2. Build release binaries
echo "==> cargo build --release"
run_as_project_user "source \"\$HOME/.cargo/env\" >/dev/null 2>&1 || true; cd \"${PROJECT_DIR}\" && cargo build --release --workspace"

# 3. Build frontend (if present)
if [[ -f "${PROJECT_DIR}/frontend/package.json" ]]; then
    echo "==> frontend build"
    run_as_project_user "cd \"${PROJECT_DIR}\" && npm --prefix frontend install && npm --prefix frontend run build"
fi

# 4. Run migrations
if [[ "${RUN_MIGRATIONS}" == "true" && -x "${PROJECT_DIR}/console" ]]; then
    echo "==> migrate"
    run_as_project_user "cd \"${PROJECT_DIR}\" && ./console migrate run"
fi

# 5. Restart supervisor processes
if [[ -n "${SUPERVISOR_PROJECT_SLUG:-}" ]] && command -v supervisorctl >/dev/null 2>&1; then
    CONF="/etc/supervisor/conf.d/${SUPERVISOR_PROJECT_SLUG}.conf"
    if [[ -f "${CONF}" ]]; then
        run_supervisorctl reread || true
        run_supervisorctl update || true

        mapfile -t programs < <(grep -oE '^\[program:[^]]+\]' "${CONF}" | sed -E 's/^\[program:([^]]+)\]$/\1/')
        for program in "${programs[@]}"; do
            [[ -z "${program}" ]] && continue
            echo "==> restart ${program}"
            run_supervisorctl restart "${program}" || run_supervisorctl start "${program}" || true
        done
    else
        echo "Supervisor config not found: ${CONF}"
    fi
fi

echo "Deploy complete."
