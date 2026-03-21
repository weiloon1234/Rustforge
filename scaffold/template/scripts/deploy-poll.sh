#!/usr/bin/env bash
set -euo pipefail

# deploy-poll.sh — Polls R2 for new versions and auto-deploys.
# Intended to run as a Supervisor-managed long-running process.
#
# Required env vars (from .env or environment):
#   PROJECT_DIR               — where the app runs (e.g., /opt/brui)
#   SUPERVISOR_PROJECT_SLUG   — supervisor program name prefix
#   S3_ENDPOINT               — R2 endpoint URL
#   S3_BUCKET                 — R2 bucket name
#   S3_ACCESS_KEY             — R2 access key ID
#   S3_SECRET_KEY             — R2 secret access key
#
# Optional:
#   DEPLOY_POLL_INTERVAL      — seconds between polls (default: 300)

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
    SUPERVISOR_PROJECT_SLUG="${SUPERVISOR_PROJECT_SLUG:-$(read_env_value "${ENV_FILE}" "SUPERVISOR_PROJECT_SLUG")}"
    PROJECT_USER="${PROJECT_USER:-$(read_env_value "${ENV_FILE}" "PROJECT_USER")}"
    S3_ENDPOINT="${S3_ENDPOINT:-$(read_env_value "${ENV_FILE}" "S3_ENDPOINT")}"
    S3_BUCKET="${S3_BUCKET:-$(read_env_value "${ENV_FILE}" "S3_BUCKET")}"
    S3_ACCESS_KEY="${S3_ACCESS_KEY:-$(read_env_value "${ENV_FILE}" "S3_ACCESS_KEY")}"
    S3_SECRET_KEY="${S3_SECRET_KEY:-$(read_env_value "${ENV_FILE}" "S3_SECRET_KEY")}"
fi

SUPERVISOR_PROJECT_SLUG="${SUPERVISOR_PROJECT_SLUG:-}"
PROJECT_USER="${PROJECT_USER:-}"
S3_ENDPOINT="${S3_ENDPOINT:-}"
S3_BUCKET="${S3_BUCKET:-}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-}"
S3_SECRET_KEY="${S3_SECRET_KEY:-}"
POLL_INTERVAL="${DEPLOY_POLL_INTERVAL:-300}"

if [[ -z "${SUPERVISOR_PROJECT_SLUG}" ]]; then
    echo "ERROR: SUPERVISOR_PROJECT_SLUG is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi
if [[ -z "${S3_ENDPOINT}" ]]; then
    echo "ERROR: S3_ENDPOINT is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi
if [[ -z "${S3_BUCKET}" ]]; then
    echo "ERROR: S3_BUCKET is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi
if [[ -z "${S3_ACCESS_KEY}" ]]; then
    echo "ERROR: S3_ACCESS_KEY is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi
if [[ -z "${S3_SECRET_KEY}" ]]; then
    echo "ERROR: S3_SECRET_KEY is not set. Set it in ${ENV_FILE} or environment."
    exit 1
fi

# Export AWS credentials for awscli
export AWS_ACCESS_KEY_ID="${S3_ACCESS_KEY}"
export AWS_SECRET_ACCESS_KEY="${S3_SECRET_KEY}"
export AWS_DEFAULT_REGION="auto"

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

s3_download() {
    local s3_path="$1"
    local local_path="$2"
    aws s3 cp "s3://${S3_BUCKET}/${s3_path}" "${local_path}" \
        --endpoint-url "${S3_ENDPOINT}" \
        --quiet 2>/dev/null
}

get_remote_version() {
    local tmp_version
    tmp_version="$(mktemp)"
    if s3_download "deploy/VERSION" "${tmp_version}"; then
        tr -d '[:space:]' < "${tmp_version}"
        rm -f "${tmp_version}"
    else
        rm -f "${tmp_version}"
        return 1
    fi
}

get_deployed_version() {
    tr -d '[:space:]' < "${PROJECT_DIR}/VERSION" 2>/dev/null || echo ""
}

deploy_new_version() {
    local version="$1"
    log "New version detected: ${version}"

    # Download artifacts to temp dir
    local download_dir
    download_dir="$(mktemp -d)"

    log "Downloading artifacts from R2..."
    if ! s3_download "deploy/${version}/release.zip" "${download_dir}/release.zip"; then
        log "ERROR: Failed to download release.zip. Aborting deploy."
        rm -rf "${download_dir}"
        return 1
    fi

    if ! s3_download "deploy/${version}/SHA256SUMS" "${download_dir}/SHA256SUMS"; then
        log "ERROR: Failed to download SHA256SUMS. Aborting deploy."
        rm -rf "${download_dir}"
        return 1
    fi

    # Verify checksum
    log "Verifying release.zip checksum..."
    if ! (cd "${download_dir}" && sha256sum -c SHA256SUMS); then
        log "ERROR: Checksum verification failed. Aborting deploy."
        rm -rf "${download_dir}"
        return 1
    fi

    # Extract to staging
    local staging
    staging="$(mktemp -d)"
    log "Extracting release.zip to staging: ${staging}"
    if ! unzip -q "${download_dir}/release.zip" -d "${staging}"; then
        log "ERROR: Failed to extract release.zip. Aborting deploy."
        rm -rf "${download_dir}" "${staging}"
        return 1
    fi

    rm -rf "${download_dir}"

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

    # Fix ownership (rsync via sudo may set root ownership)
    if [[ -n "${PROJECT_USER}" ]]; then
        chown -R "${PROJECT_USER}:${PROJECT_USER}" "${PROJECT_DIR}"
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

# Main poll loop
log "Deploy poller started."
log "  PROJECT_DIR: ${PROJECT_DIR}"
log "  S3_ENDPOINT: ${S3_ENDPOINT}"
log "  S3_BUCKET: ${S3_BUCKET}"
log "  SUPERVISOR_PROJECT_SLUG: ${SUPERVISOR_PROJECT_SLUG}"
log "  POLL_INTERVAL: ${POLL_INTERVAL}s"

while true; do
    # Fetch remote version from R2
    REMOTE_VERSION="$(get_remote_version 2>/dev/null || true)"
    DEPLOYED_VERSION="$(get_deployed_version)"

    if [[ -z "${REMOTE_VERSION}" ]]; then
        log "WARNING: Could not fetch VERSION from R2. Will retry next cycle."
        sleep "${POLL_INTERVAL}"
        continue
    fi

    if [[ "${REMOTE_VERSION}" == "${DEPLOYED_VERSION}" ]]; then
        sleep "${POLL_INTERVAL}"
        continue
    fi

    if deploy_new_version "${REMOTE_VERSION}"; then
        log "Successfully deployed ${REMOTE_VERSION}"
    else
        log "Deploy failed for ${REMOTE_VERSION}. Will retry next cycle."
    fi

    sleep "${POLL_INTERVAL}"
done
