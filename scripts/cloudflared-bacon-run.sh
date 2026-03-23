#!/usr/bin/env bash
#
# dev.sh — Start a cloudflared tunnel and run the dev server via bacon.
#
# SITE_URL is set from the tunnel URL so that og:image tags in rendered pages
# resolve to a publicly reachable address, enabling social-card preview testing
# from LinkedIn, Slack, or X while developing locally.
#
# Usage:
#   ./scripts/dev.sh
#
# In a separate terminal, also run:
#   ./scripts/tailwatch.sh
#
# To test social cards:
#   1. Copy the tunnel URL printed below.
#   2. Append a post path, e.g. https://<tunnel>.trycloudflare.com/posts/<slug>
#   3. Paste that URL into the relevant inspector:
#        LinkedIn  — https://www.linkedin.com/post-inspector/
#        Slack     — paste the URL into any channel; Slack fetches og:image live
#        X/Twitter — https://cards-dev.twitter.com/validator
#   Note: inspectors cache aggressively. If the card looks stale, append ?v=2
#   (or any query param) to bust the cache.

set -euo pipefail

# ── Dependency checks ──────────────────────────────────────────────────────────

if ! command -v cloudflared &>/dev/null; then
    echo "Error: 'cloudflared' not found in PATH." >&2
    echo "  Install: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/" >&2
    exit 1
fi

if ! command -v cargo &>/dev/null; then
    echo "Error: 'cargo' not found in PATH." >&2
    echo "  Install: cargo install cargo" >&2
    exit 1
fi

# ── Start cloudflared ──────────────────────────────────────────────────────────

TUNNEL_LOG=$(mktemp)
cloudflared tunnel --url http://localhost:8080 2>"$TUNNEL_LOG" &
CLOUDFLARED_PID=$!

cleanup() {
    kill "$CLOUDFLARED_PID" 2>/dev/null || true
    rm -f "$TUNNEL_LOG"
}
trap cleanup EXIT

# ── Wait for tunnel URL ────────────────────────────────────────────────────────

echo "Starting cloudflared tunnel..."
SITE_URL=""
for i in $(seq 1 30); do
    SITE_URL=$(grep -Eo 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' "$TUNNEL_LOG" 2>/dev/null | head -1 || true)
    [ -n "$SITE_URL" ] && break
    sleep 1
done

if [ -z "$SITE_URL" ]; then
    echo "Error: timed out waiting for cloudflared URL after 30 seconds." >&2
    echo "" >&2
    echo "cloudflared output:" >&2
    cat "$TUNNEL_LOG" >&2
    exit 1
fi

# ── Print tunnel info ──────────────────────────────────────────────────────────

echo ""
echo "  Tunnel: $SITE_URL"
echo ""
echo "  Test social cards:"
echo "    LinkedIn  — https://www.linkedin.com/post-inspector/"
echo "    Slack     — paste \$SITE_URL/posts/<slug> into any channel"
echo "    X/Twitter — https://cards-dev.twitter.com/validator"
echo ""

# ── Run cargo ──────────────────────────────────────────────────────────────────

export SITE_URL
RUST_ENV=development cargo run
