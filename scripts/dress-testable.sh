#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

: "${GCP_SERVICE_NAME:?Set GCP_SERVICE_NAME}"

sanitize_name() {
  printf '%s' "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | tr -c 'a-z0-9-' '-' \
    | sed -e 's/--*/-/g' -e 's/^-//' -e 's/-$//'
}

RUN_SUFFIX="${DRESS_NAME_SUFFIX:-$(date -u +%Y%m%d%H%M%S)-${RANDOM}}"
NAME_PREFIX="$(sanitize_name "${GCP_SERVICE_NAME}-dr-${RUN_SUFFIX}")"
NAME_PREFIX="${NAME_PREFIX:0:24}"
NAME_PREFIX="${NAME_PREFIX%-}"

export DRESS_DEPLOYMENT_ROOT="${DRESS_DEPLOYMENT_ROOT:-${ROOT_DIR}/infra/testable}"
export DRESS_TERRAFORM_BINARY="${DRESS_TERRAFORM_BINARY:-tofu}"

export TF_VAR_repository_id="${NAME_PREFIX}"
export TF_VAR_service_name="${NAME_PREFIX}"
export TF_VAR_load_balancer_ip_name="${NAME_PREFIX}-ip"
export TF_VAR_network_endpoint_group_name="${NAME_PREFIX}-neg"
export TF_VAR_backend_service_name="${NAME_PREFIX}-backend"
export TF_VAR_url_map_name="${NAME_PREFIX}-url-map"
export TF_VAR_https_proxy_name="${NAME_PREFIX}-https-proxy"
export TF_VAR_https_forwarding_rule_name="${NAME_PREFIX}-https-fr"
export TF_VAR_http_redirect_url_map_name="${NAME_PREFIX}-http-redirect"
export TF_VAR_http_proxy_name="${NAME_PREFIX}-http-proxy"
export TF_VAR_http_forwarding_rule_name="${NAME_PREFIX}-http-fr"

exec dress "$@"
