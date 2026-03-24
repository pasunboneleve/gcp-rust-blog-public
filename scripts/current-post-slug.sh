#!/usr/bin/env bash
set -euo pipefail

port="${PORT:-8080}"
current_path="$(curl -fsS --max-time 1 "http://127.0.0.1:${port}/__dev/current-path" 2>/dev/null || true)"
if [[ "${current_path}" =~ ^/posts/([a-z0-9-]+)$ ]]; then
  printf '%s\n' "${BASH_REMATCH[1]}"
  exit 0
fi

find "${DEVLOOP_ROOT:-.}/content/posts" -maxdepth 1 -type f -name '*.md' -printf '%f\n' \
  | sort \
  | tail -n1 \
  | sed 's/\.md$//'
