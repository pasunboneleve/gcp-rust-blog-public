#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://localhost:8080}"
PATH_PART="${2:-/}"

if [[ "$PATH_PART" != /* ]]; then
  PATH_PART="/$PATH_PART"
fi

URL="${BASE_URL%/}${PATH_PART}"

if ! command -v rg >/dev/null 2>&1; then
  echo "Error: ripgrep (rg) is required." >&2
  exit 1
fi

echo "Checking social preview tags at: $URL"
HTML="$(curl -fsSL "$URL")"

if echo "$HTML" | rg -q '\{\{\s*page_(title|description|url|image)\s*\}\}'; then
  echo "FAIL: unresolved page metadata placeholders found in HTML." >&2
  exit 2
fi

REQUIRED_PATTERNS=(
  '<title>'
  'meta name="description"'
  'meta property="og:title"'
  'meta property="og:description"'
  'meta property="og:url"'
  'meta property="og:image"'
  'meta name="twitter:title"'
  'meta name="twitter:description"'
  'meta name="twitter:image"'
)

missing=0
for pattern in "${REQUIRED_PATTERNS[@]}"; do
  if ! echo "$HTML" | rg -q "$pattern"; then
    echo "MISSING: $pattern"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "FAIL: one or more required social tags are missing." >&2
  exit 3
fi

echo
echo "Extracted tag lines:"
echo "$HTML" | rg -n '<title>|meta name="description"|meta property="og:(title|description|url|image)"|meta name="twitter:(title|description|image)"'

echo
echo "OK: social preview tags look good."
