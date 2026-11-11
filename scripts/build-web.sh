#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Required command '$1' was not found. See README.md for browser build instructions." >&2
    exit 1
  fi
}

require_command cargo
require_command rustup

FONT_DIR="$ROOT/assets/fonts"
FONT_PATH="$FONT_DIR/NotoSansMono-VF.ttf"
FONT_LICENSE_PATH="$FONT_DIR/OFL.txt"
FONT_URL="https://raw.githubusercontent.com/google/fonts/main/ofl/notosansmono/NotoSansMono%5Bwdth%2Cwght%5D.ttf"
FONT_LICENSE_URL="https://raw.githubusercontent.com/google/fonts/main/ofl/notosansmono/OFL.txt"
mkdir -p "$FONT_DIR"

if [ ! -s "$FONT_PATH" ] || [ "$(wc -c < "$FONT_PATH")" -lt 10000 ]; then
  echo "Downloading Noto Sans Mono for WebAssembly text rendering..."
  FONT_TEMP="$FONT_PATH.tmp"
  if command -v curl >/dev/null 2>&1; then
    curl --fail --location --retry 3 --output "$FONT_TEMP" "$FONT_URL"
  elif command -v wget >/dev/null 2>&1; then
    wget --tries=3 --output-document="$FONT_TEMP" "$FONT_URL"
  else
    echo "curl or wget is required to download the web font." >&2
    exit 1
  fi
  if [ ! -s "$FONT_TEMP" ] || [ "$(wc -c < "$FONT_TEMP")" -lt 10000 ]; then
    echo "The Noto Sans Mono download is missing or incomplete." >&2
    rm -f "$FONT_TEMP"
    exit 1
  fi
  mv "$FONT_TEMP" "$FONT_PATH"
fi

if [ ! -s "$FONT_LICENSE_PATH" ]; then
  echo "Downloading the Noto Sans Mono OFL license..."
  LICENSE_TEMP="$FONT_LICENSE_PATH.tmp"
  if command -v curl >/dev/null 2>&1; then
    curl --fail --location --retry 3 --output "$LICENSE_TEMP" "$FONT_LICENSE_URL"
  else
    wget --tries=3 --output-document="$LICENSE_TEMP" "$FONT_LICENSE_URL"
  fi
  if [ ! -s "$LICENSE_TEMP" ] || [ "$(wc -c < "$LICENSE_TEMP")" -lt 1000 ]; then
    echo "The Noto Sans Mono license download is missing or incomplete." >&2
    rm -f "$LICENSE_TEMP"
    exit 1
  fi
  mv "$LICENSE_TEMP" "$FONT_LICENSE_PATH"
fi

if ! rustup target list --installed | grep -qx 'wasm32-unknown-unknown'; then
  echo "Installing the required browser target..."
  rustup target add wasm32-unknown-unknown
fi

if [ ! -f Cargo.lock ]; then
  echo "Generating Cargo.lock..."
  cargo generate-lockfile
fi

EXPECTED_BINDGEN_VERSION="$(awk '
  /^\[\[package\]\]$/ { in_package = 0 }
  /^name = "wasm-bindgen"$/ { in_package = 1; next }
  in_package && /^version = "/ {
    gsub(/^version = "/, "")
    gsub(/"$/, "")
    print
    exit
  }
' Cargo.lock)"
if [ -z "$EXPECTED_BINDGEN_VERSION" ]; then
  echo "Could not find wasm-bindgen in Cargo.lock. Check the dependency tree." >&2
  exit 1
fi

INSTALLED_BINDGEN_VERSION=""
if command -v wasm-bindgen >/dev/null 2>&1; then
  INSTALLED_BINDGEN_VERSION="$(wasm-bindgen --version | awk '{print $2}')"
fi
if [ "$INSTALLED_BINDGEN_VERSION" != "$EXPECTED_BINDGEN_VERSION" ]; then
  echo "Installing matching wasm-bindgen-cli $EXPECTED_BINDGEN_VERSION (current: ${INSTALLED_BINDGEN_VERSION:-not installed})..."
  cargo install wasm-bindgen-cli --version "$EXPECTED_BINDGEN_VERSION" --locked --force
fi

rm -rf web-dist
mkdir -p web-dist/pkg/webgl2 web-dist/pkg/webgpu

build_renderer() {
  renderer="$1"
  feature_args="$2"
  output_dir="web-dist/pkg/$renderer"

  echo "Building Retro Pong WebAssembly renderer: $renderer"
  renderer_target_dir="$ROOT/target/web-$renderer"
  if [ -n "$feature_args" ]; then
    CARGO_TARGET_DIR="$renderer_target_dir" cargo build --release --target wasm32-unknown-unknown $feature_args
  else
    CARGO_TARGET_DIR="$renderer_target_dir" cargo build --release --target wasm32-unknown-unknown
  fi

  wasm_artifact="$(find "$renderer_target_dir/wasm32-unknown-unknown/release" -maxdepth 1 -type f -name '*.wasm' -print -quit)"
  if [ -z "$wasm_artifact" ] || [ ! -s "$wasm_artifact" ]; then
    echo "Could not find a compiled .wasm executable for renderer '$renderer'." >&2
    exit 1
  fi

  wasm-bindgen \
    --target web \
    --out-dir "$output_dir" \
    --out-name retro_pong \
    "$wasm_artifact"

  if [ ! -s "$output_dir/retro_pong.js" ] || [ ! -s "$output_dir/retro_pong_bg.wasm" ]; then
    echo "WebAssembly output for '$renderer' is incomplete." >&2
    exit 1
  fi
}

build_renderer webgl2 ""
build_renderer webgpu "--no-default-features --features renderer-webgpu"

cp web/index.html web-dist/index.html
cp web/style.css web-dist/style.css
mkdir -p web-dist/assets
cp -R assets/. web-dist/assets/
if [ ! -s "web-dist/assets/fonts/NotoSansMono-VF.ttf" ] || [ "$(wc -c < "web-dist/assets/fonts/NotoSansMono-VF.ttf")" -lt 10000 ]; then
  echo "The bundled WebAssembly font is missing from web-dist/assets/fonts." >&2
  exit 1
fi
: > web-dist/.nojekyll

printf '\nWeb build completed: %s/web-dist\n' "$ROOT"
printf 'Generated renderers: WebGL2 (preferred) and WebGPU (fallback).\n'
printf 'Run ./scripts/serve-web.sh to preview it locally.\n'
