#!/usr/bin/env bash
# Build the Bevy game for the web and stage a GitHub Pages-ready directory.
#
# Usage: tools/build_web.sh [output-dir]   (default: web/dist)
# Needs: rustup target add wasm32-unknown-unknown
#        wasm-bindgen-cli matching Cargo.lock
# Optional: wasm-opt (Binaryen) for a smaller bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${1:-$ROOT/web/dist}"
WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
PROFILE_DIR="$ROOT/target/wasm32-unknown-unknown/wasm-release"
WASM="$PROFILE_DIR/spy-hunter.wasm"
OUT_NAME="spy_hunter"

cd "$ROOT"

LOCK_VERSION="$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/,""); print; exit}' Cargo.lock)"
CLI_VERSION="$("$WASM_BINDGEN" --version | awk '{print $2}')"
if [[ -n "$LOCK_VERSION" && "$LOCK_VERSION" != "$CLI_VERSION" ]]; then
  echo "wasm-bindgen mismatch: Cargo.lock has $LOCK_VERSION, CLI is $CLI_VERSION" >&2
  exit 1
fi

echo "==> building wasm (profile: wasm-release)"
cargo build --profile wasm-release --target wasm32-unknown-unknown --bin spy-hunter

if [[ ! -f "$WASM" ]]; then
  echo "wasm artifact not found at $WASM" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST/assets"

echo "==> wasm-bindgen"
"$WASM_BINDGEN" \
  --target web \
  --no-typescript \
  --out-name "$OUT_NAME" \
  --out-dir "$DIST" \
  "$WASM"

if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> wasm-opt -Oz"
  wasm-opt -Oz --output "$DIST/${OUT_NAME}_bg.opt.wasm" "$DIST/${OUT_NAME}_bg.wasm"
  mv "$DIST/${OUT_NAME}_bg.opt.wasm" "$DIST/${OUT_NAME}_bg.wasm"
else
  echo "==> wasm-opt not found; shipping the unoptimised wasm"
fi

echo "==> staging static shell and runtime assets"
cp "$ROOT/web/index.html" "$ROOT/web/style.css" "$ROOT/web/favicon.svg" "$DIST/"
cp -R "$ROOT/assets/sprites" "$ROOT/assets/backgrounds" "$ROOT/assets/audio" "$ROOT/assets/fonts" "$DIST/assets/"
mkdir -p "$DIST/assets/art"
find "$ROOT/assets/art" -maxdepth 1 -type f \( -name '*.png' -o -name '*.webp' \) -exec cp {} "$DIST/assets/art/" \;
touch "$DIST/.nojekyll"

echo
echo "built: $DIST"
du -sh "$DIST"
find "$DIST" -maxdepth 1 -type f -printf '  %-28f %10s bytes\n' | sort
