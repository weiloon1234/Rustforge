#!/usr/bin/env bash
set -euo pipefail

# deploy-poll.sh — Polls the deploy repo for new versions and auto-deploys.
# Intended to run as a Supervisor-managed long-running process.
#
# Required env vars (from .env or environment):
#   PROJECT_DIR           — where the app runs (e.g., /opt/your_project)
#   DEPLOY_REPO_DIR       — local clone of your_company/your_project-deploy
#   SUPERVISOR_PROJECT_SLUG — supervisor program name prefix
#
# Optional:
#   DEPLOY_POLL_INTERVAL  — seconds between polls (default: 300)

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

# Resolve PROJECT_DIR: prefer env var, fall back to parent of script dir
PROJECT_DIR="${PROJECT_DIR:-$(cd "${SCRIPT_DIR}/.." >/dev/null 2>&1 && pwd)}"
ENV_FILE="${PROJECT_DIR}/.env"

read_env_value() {
    local file="$1"
    local key="$2"
    [[ -f "${file}" ]] || return 0
    awk -F= -v k="${key}" '
        $1 == k {
            sub(/^[[:space:]]+/, "", $2)
            sub(/[[:space:]]+$/, "", $2)
            print $2
            exit
        }
    ' "${file}"
}

# Load config from .env
if [[ -f "${ENV_FILE}" ]]; then
    DEPLOY_REPO_DIR="${DEPLOY_REPO_DIR:-$(read_env_value "${ENV_FILE}" "DEPLOY_REPO_DIR")}"
    SUPERVISOR_PROJECT_SLUG="${SUPERVISOR_PROJECT_SLUG:-$(read_env_value "${ENV_FILE}" "SUPERVISOR_PROJECT_SLUG")}"
    PROJECT_USER="${PROJECT_USER:-$(read_env_value "${ENV_FILE}" "PROJECT_USER")}"
fi

DEPLOY_REPO_DIR="${DEPLOY_REPO_DIR:-}"
SUPERVISOR_PROJECT_SLUG="${SUPERVISOR_PROJECT_SLUG:-}"
PROJECT_USER="${PROJECT_USER:-}"
POLL_INTERVAL="${DEPLOY_POLL_INTERVAL:-300}"

if [[ -z "${DEPLOY_REPO_DIR}" ]]; then
    echo "ERROR: DEPLOY_REPO_DIR is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi
if [[ -z "${SUPERVISOR_PROJECT_SLUG}" ]]; then
    echo "ERROR: SUPERVISOR_PROJECT_SLUG is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi

log() {
    echo "[$(date -u '+%Y-%m-%d %H:%M:%S UTC')] $*"
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

run_as_project_user() {
    local command="$1"
    if [[ -n "${PROJECT_USER}" && "$(id -u)" -eq 0 ]]; then
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

deploy_new_version() {
    local version="$1"
    log "New version detected: ${version}"

    # Verify checksum
    log "Verifying release.zip checksum..."
    cd "${DEPLOY_REPO_DIR}"
    if ! sha256sum -c SHA256SUMS; then
        log "ERROR: Checksum verification failed. Aborting deploy."
        return 1
    fi

    # Extract to staging
    local staging
    staging="$(mktemp -d)"
    log "Extracting release.zip to staging: ${staging}"
    if ! unzip -q "${DEPLOY_REPO_DIR}/release.zip" -d "${staging}"; then
        log "ERROR: Failed to extract release.zip. Aborting deploy."
        rm -rf "${staging}"
        return 1
    fi

    # Rsync to project directory, preserving .env and logs
    log "Syncing files to ${PROJECT_DIR}..."
    if ! rsync -a --delete \
        --exclude='.env' \
        --exclude='logs/' \
        --exclude='.git/' \
        "${staging}/" "${PROJECT_DIR}/"; then
        log "ERROR: rsync failed. Aborting deploy."
        rm -rf "${staging}"
        return 1
    fi

    # Fix permissions on binaries
    chmod +x "${PROJECT_DIR}/target/release/api-server" \
              "${PROJECT_DIR}/target/release/websocket-server" \
              "${PROJECT_DIR}/target/release/worker" \
              "${PROJECT_DIR}/target/release/console" \
              "${PROJECT_DIR}/bin/api-server" \
              "${PROJECT_DIR}/bin/websocket-server" \
              "${PROJECT_DIR}/bin/worker" \
              "${PROJECT_DIR}/bin/console" \
              "${PROJECT_DIR}/console" \
              "${PROJECT_DIR}/scripts/"*.sh 2>/dev/null || true

    rm -rf "${staging}"

    # Run migrations
    log "Running migrations..."
    if ! run_as_project_user "cd \"${PROJECT_DIR}\" && ./console migrate run"; then
        log "ERROR: Migration failed! Services will NOT be restarted."
        log "Fix the migration issue and manually restart services."
        return 1
    fi

    # Sequential restart: worker -> ws -> api
    local slug="${SUPERVISOR_PROJECT_SLUG}"
    log "Restarting services..."

    log "  Restarting ${slug}-worker..."
    run_supervisorctl restart "${slug}-worker" 2>/dev/null || true

    log "  Restarting ${slug}-ws..."
    run_supervisorctl restart "${slug}-ws" 2>/dev/null || true

    log "  Restarting ${slug}-api..."
    run_supervisorctl restart "${slug}-api" 2>/dev/null || true

    log "Deploy complete: ${version}"
}

get_deployed_version() {
    cat "${PROJECT_DIR}/VERSION" 2>/dev/null || echo ""
}

get_repo_version() {
    cat "${DEPLOY_REPO_DIR}/VERSION" 2>/dev/null || echo ""
}

# Main poll loop
log "Deploy poller started."
log "  PROJECT_DIR: ${PROJECT_DIR}"
log "  DEPLOY_REPO_DIR: ${DEPLOY_REPO_DIR}"
log "  SUPERVISOR_PROJECT_SLUG: ${SUPERVISOR_PROJECT_SLUG}"
log "  POLL_INTERVAL: ${POLL_INTERVAL}s"

while true; do
    cd "${DEPLOY_REPO_DIR}"

    # Fetch and pull latest from remote
    if ! git fetch origin main --quiet 2>/dev/null; then
        log "WARNING: git fetch failed. Will retry next cycle."
        sleep "${POLL_INTERVAL}"
        continue
    fi

    LOCAL_HEAD="$(git rev-parse HEAD)"
    REMOTE_HEAD="$(git rev-parse origin/main)"

    if [[ "${LOCAL_HEAD}" != "${REMOTE_HEAD}" ]]; then
        if ! git pull origin main --quiet 2>/dev/null; then
            log "WARNING: git pull failed. Will retry next cycle."
            sleep "${POLL_INTERVAL}"
            continue
        fi
    fi

    # Compare deployed version vs repo version (not git HEAD)
    # This ensures failed deploys are retried on next cycle
    REPO_VERSION="$(get_repo_version)"
    DEPLOYED_VERSION="$(get_deployed_version)"

    if [[ "${REPO_VERSION}" == "${DEPLOYED_VERSION}" ]]; then
        sleep "${POLL_INTERVAL}"
        continue
    fi

    if [[ -z "${REPO_VERSION}" ]]; then
        log "WARNING: No VERSION file in deploy repo. Skipping."
        sleep "${POLL_INTERVAL}"
        continue
    fi

    if deploy_new_version "${REPO_VERSION}"; then
        log "Successfully deployed ${REPO_VERSION}"
    else
        log "Deploy failed for ${REPO_VERSION}. Will retry next cycle."
    fi

    sleep "${POLL_INTERVAL}"
done
