#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
WEB_DIR="$ROOT/web-dist"
PORT="${1:-${PORT:-8000}}"

if [ ! -f "$WEB_DIR/index.html" ]; then
  echo "web-dist/index.html was not found. Run ./scripts/build-web.sh first." >&2
  exit 1
fi

if [ ! -s "$WEB_DIR/assets/fonts/NotoSansMono-VF.ttf" ]; then
  echo "The WebAssembly font is missing from web-dist. Run ./scripts/build-web.sh again." >&2
  exit 1
fi

if command -v python3 >/dev/null 2>&1; then
  PYTHON=python3
elif command -v python >/dev/null 2>&1; then
  PYTHON=python
else
  echo "Python 3 is required to serve the web build locally." >&2
  exit 1
fi

exec "$PYTHON" -m http.server "$PORT" --bind 127.0.0.1 --directory "$WEB_DIR"
