#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo test core::subsystem_registry::tests:: --lib

echo "Core subsystem registry guard passed."
