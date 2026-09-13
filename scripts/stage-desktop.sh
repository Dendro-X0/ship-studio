#!/usr/bin/env bash
# Stage portable desktop pair: ship-studio-desktop.exe + shipctl.exe side by side.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo build -p shipctl --release"
cargo build -p shipctl --release

echo "==> tauri build --no-bundle"
cd apps/desktop
export COREPACK_ENABLE=0
./node_modules/.bin/tauri build --no-bundle
cd "$ROOT"

DESKTOP="$ROOT/target/release/ship-studio-desktop.exe"
SHIPCTL="$ROOT/target/release/shipctl.exe"
if [[ ! -f "$DESKTOP" ]]; then
  DESKTOP="$ROOT/target/release/ship-studio-desktop"
  SHIPCTL="$ROOT/target/release/shipctl"
fi

DESKTOP_DIR="$(dirname "$DESKTOP")"
if [[ "$(cd "$(dirname "$SHIPCTL")" && pwd)/$(basename "$SHIPCTL")" != "$(cd "$DESKTOP_DIR" && pwd)/$(basename "$SHIPCTL")" ]]; then
  cp -f "$SHIPCTL" "$DESKTOP_DIR/"
else
  echo "==> shipctl already beside desktop exe"
fi
echo "==> staged:"
ls -la "$DESKTOP" "$DESKTOP_DIR/$(basename "$SHIPCTL")" 2>/dev/null || true
echo "Run: $DESKTOP"
