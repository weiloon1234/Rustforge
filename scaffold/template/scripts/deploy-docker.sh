#!/usr/bin/env bash
set -euo pipefail

# deploy-docker.sh — Build release.zip in Docker and upload to R2.
# Mirrors the GitHub Actions deploy.yml pipeline exactly.
#
# Usage:
#   ./scripts/deploy-docker.sh <version-tag> <environment>
#   ./scripts/deploy-docker.sh v1.2.3 staging
#   ./scripts/deploy-docker.sh v1.2.3 production
#
# R2 layout:
#   deploy/<environment>/<version>/release.zip
#   deploy/<environment>/<version>/SHA256SUMS
#   deploy/<environment>/VERSION
#
# Reads R2 credentials and VITE_* vars from .env (same file as deploy-poll.sh).

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." >/dev/null 2>&1 && pwd)"
ENV_FILE="${PROJECT_DIR}/.env"

# ── Parse arguments ───────────────────────────────────────────────────
VERSION="${1:-}"
DEPLOY_ENV="${2:-staging}"

if [[ -z "${VERSION}" ]]; then
    echo "Usage: $0 <version-tag> [environment]"
    echo "  e.g. $0 v1.2.3 staging"
    echo "  e.g. $0 v1.2.3 production"
    exit 1
fi

if [[ ! "${VERSION}" =~ ^v[0-9] ]]; then
    echo "ERROR: Version tag must start with 'v' followed by a digit (e.g., v1.0.0)."
    exit 1
fi

if [[ "${DEPLOY_ENV}" != "production" && "${DEPLOY_ENV}" != "staging" ]]; then
    echo "ERROR: Environment must be 'production' or 'staging' (got '${DEPLOY_ENV}')."
    exit 1
fi

# R2 prefix for this environment
R2_PREFIX="deploy/${DEPLOY_ENV}"

# ── Read .env helper ──────────────────────────────────────────────────
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

# ── Load config from .env ─────────────────────────────────────────────
if [[ -f "${ENV_FILE}" ]]; then
    S3_ENDPOINT="${S3_ENDPOINT:-$(read_env_value "${ENV_FILE}" "S3_ENDPOINT")}"
    S3_BUCKET="${S3_BUCKET:-$(read_env_value "${ENV_FILE}" "S3_BUCKET")}"
    S3_ACCESS_KEY="${S3_ACCESS_KEY:-$(read_env_value "${ENV_FILE}" "S3_ACCESS_KEY")}"
    S3_SECRET_KEY="${S3_SECRET_KEY:-$(read_env_value "${ENV_FILE}" "S3_SECRET_KEY")}"
    VITE_APP_NAME="${VITE_APP_NAME:-$(read_env_value "${ENV_FILE}" "VITE_APP_NAME")}"
    VITE_S3_URL="${VITE_S3_URL:-$(read_env_value "${ENV_FILE}" "VITE_S3_URL")}"
else
    echo "WARNING: .env not found at ${ENV_FILE}"
fi

S3_ENDPOINT="${S3_ENDPOINT:-}"
S3_BUCKET="${S3_BUCKET:-}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-}"
S3_SECRET_KEY="${S3_SECRET_KEY:-}"
VITE_APP_NAME="${VITE_APP_NAME:-}"
VITE_S3_URL="${VITE_S3_URL:-}"

for var in S3_ENDPOINT S3_BUCKET S3_ACCESS_KEY S3_SECRET_KEY; do
    if [[ -z "${!var}" ]]; then
        echo "ERROR: ${var} is not set. Set it in ${ENV_FILE} or environment."
        exit 1
    fi
done

export AWS_ACCESS_KEY_ID="${S3_ACCESS_KEY}"
export AWS_SECRET_ACCESS_KEY="${S3_SECRET_KEY}"
export AWS_DEFAULT_REGION="auto"

# ── Docker build ──────────────────────────────────────────────────────
IMAGE_TAG="deploy-build:${VERSION}"
WORK_DIR="$(mktemp -d)"

echo "==> Building release artifacts in Docker..."
echo "    Version     : ${VERSION}"
echo "    Environment : ${DEPLOY_ENV}"
echo "    Image       : ${IMAGE_TAG}"
echo

docker build -f "${PROJECT_DIR}/Dockerfile.deploy" \
    --build-arg VERSION="${VERSION}" \
    --build-arg VITE_APP_NAME="${VITE_APP_NAME}" \
    --build-arg VITE_S3_URL="${VITE_S3_URL}" \
    -t "${IMAGE_TAG}" \
    "${PROJECT_DIR}"

# ── Extract artifacts ─────────────────────────────────────────────────
echo "==> Extracting release.zip..."
CONTAINER_ID=$(docker create "${IMAGE_TAG}")
docker cp "${CONTAINER_ID}:/out/release.zip" "${WORK_DIR}/release.zip"
docker cp "${CONTAINER_ID}:/out/SHA256SUMS" "${WORK_DIR}/SHA256SUMS"
docker rm "${CONTAINER_ID}" > /dev/null

echo "    release.zip : $(du -h "${WORK_DIR}/release.zip" | cut -f1)"

# Verify checksum locally
echo "==> Verifying checksum..."
(cd "${WORK_DIR}" && sha256sum -c SHA256SUMS)

# ── Upload to R2 ─────────────────────────────────────────────────────
echo "==> Uploading to R2 (${DEPLOY_ENV})..."

# Versioned artifacts
aws s3 cp "${WORK_DIR}/release.zip" \
    "s3://${S3_BUCKET}/${R2_PREFIX}/${VERSION}/release.zip" \
    --endpoint-url "${S3_ENDPOINT}"

aws s3 cp "${WORK_DIR}/SHA256SUMS" \
    "s3://${S3_BUCKET}/${R2_PREFIX}/${VERSION}/SHA256SUMS" \
    --endpoint-url "${S3_ENDPOINT}"

# Update VERSION pointer for this environment
echo "${VERSION}" > "${WORK_DIR}/VERSION"
aws s3 cp "${WORK_DIR}/VERSION" \
    "s3://${S3_BUCKET}/${R2_PREFIX}/VERSION" \
    --endpoint-url "${S3_ENDPOINT}"

# ── Cleanup old versions (retain 3 per environment) ──────────────────
echo "==> Cleaning old ${DEPLOY_ENV} versions (keeping 3 latest)..."
PREFIXES=$(aws s3api list-objects-v2 \
    --bucket "${S3_BUCKET}" \
    --prefix "${R2_PREFIX}/v" \
    --delimiter "/" \
    --endpoint-url "${S3_ENDPOINT}" \
    --query "CommonPrefixes[].Prefix" \
    --output text 2>/dev/null || true)

if [[ -n "${PREFIXES}" ]]; then
    echo "${PREFIXES}" | tr '\t' '\n' | sort -r | tail -n +4 | while read -r prefix; do
        if [[ -n "${prefix}" ]]; then
            echo "    Deleting: ${prefix}"
            aws s3 rm "s3://${S3_BUCKET}/${prefix}" --recursive --endpoint-url "${S3_ENDPOINT}"
        fi
    done
fi

# ── Cleanup Docker image (keep build cache for fast incremental builds) ──
echo "==> Removing build image (build cache retained)..."
docker rmi "${IMAGE_TAG}" 2>/dev/null || true

# ── Cleanup temp ──────────────────────────────────────────────────────
rm -rf "${WORK_DIR}"

echo
echo "Deploy complete: ${VERSION} → ${DEPLOY_ENV}"
echo "  R2 path: s3://${S3_BUCKET}/${R2_PREFIX}/${VERSION}/release.zip"
