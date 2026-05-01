#!/usr/bin/env bash
# Outbound HTTP client guard.
#
# H19 requires shared outbound HTTP clients instead of ad hoc reqwest clients.
# Keep the default threshold at zero; use the environment override only when
# documenting an explicit, time-boxed exception.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

MAX_ALLOWED="${LITELLM_OUTBOUND_CLIENT_BASELINE_MAX:-0}"

if ! command -v rg >/dev/null 2>&1; then
    echo "Outbound HTTP client guard failed: 'rg' is required." >&2
    exit 1
fi

matches="$(
    rg -n --no-heading --color never \
        -g '!**/*test*' \
        -g '!**/tests.rs' \
        'reqwest::Client::new\(\)' \
        src/ || true
)"

count="$(printf '%s\n' "$matches" | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ')"

echo "Outbound HTTP client guard: $count reqwest::Client::new() hits (max: $MAX_ALLOWED)."

if [[ "$count" -gt "$MAX_ALLOWED" ]]; then
    echo "FAIL: outbound HTTP client count exceeds the allowed maximum."
    printf '%s\n' "$matches"
    echo
    echo "Use src/core/http/outbound.rs helpers, or set LITELLM_OUTBOUND_CLIENT_BASELINE_MAX only for a documented exception."
    exit 1
fi

exit 0
