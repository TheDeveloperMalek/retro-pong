#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if ! command -v cargo >/dev/null 2>&1; then
  echo "The required development tools are missing or are not on PATH." >&2
  echo "Install the project prerequisites once, then run this script again." >&2
  exit 1
fi
exec cargo run --release
