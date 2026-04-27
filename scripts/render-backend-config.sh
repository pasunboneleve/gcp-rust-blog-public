#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INFRA_DIR="${ROOT_DIR}/infra"
TF_BACKEND_CONFIG_OUTPUT="${TF_BACKEND_CONFIG_OUTPUT:-${INFRA_DIR}/backend.auto.hcl}"

require_env() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "Missing required environment variable: ${name}" >&2
    exit 1
  fi
}

hcl_string() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '"%s"' "${value}"
}

require_env GCS_BUCKET

GCP_TF_STATE_PREFIX="${GCP_TF_STATE_PREFIX:-gcp-rust-blog/infra}"

mkdir -p "$(dirname "${TF_BACKEND_CONFIG_OUTPUT}")"

cat >"${TF_BACKEND_CONFIG_OUTPUT}" <<EOF
bucket = $(hcl_string "${GCS_BUCKET}")
prefix = $(hcl_string "${GCP_TF_STATE_PREFIX}")
EOF
