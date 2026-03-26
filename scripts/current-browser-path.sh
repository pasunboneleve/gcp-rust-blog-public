#!/usr/bin/env bash
set -euo pipefail

port="${PORT:-8080}"
current_path="$(curl -fsS --max-time 1 "http://127.0.0.1:${port}/__dev/current-path" 2>/dev/null || true)"
if [[ "${current_path}" =~ ^/ ]]; then
  printf '%s\n' "${current_path}"
  exit 0
fi

latest_slug="$(
  find "${DEVLOOP_ROOT:-.}/content/posts" -maxdepth 1 -type f -name '*.md' -printf '%f\n' \
    | sort \
    | tail -n1 \
    | sed 's/\.md$//'
)"

if [[ -n "${latest_slug}" ]]; then
  printf '/posts/%s\n' "${latest_slug}"
else
  printf '/\n'
fi
