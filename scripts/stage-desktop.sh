#!/usr/bin/env bash
# Stage portable desktop pair: ship-studio-desktop.exe + shipctl.exe side by side.
# With --installer: also build NSIS setup.exe (bundles resources/shipctl.exe).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BUNDLE_INSTALLER=0
for arg in "$@"; do
  case "$arg" in
    --installer) BUNDLE_INSTALLER=1 ;;
  esac
done

echo "==> cargo build -p shipctl --release"
cargo build -p shipctl --release

SHIPCTL_SRC="$ROOT/target/release/shipctl.exe"
if [[ ! -f "$SHIPCTL_SRC" ]]; then
  SHIPCTL_SRC="$ROOT/target/release/shipctl"
fi
mkdir -p apps/desktop/src-tauri/resources
cp -f "$SHIPCTL_SRC" apps/desktop/src-tauri/resources/shipctl.exe
echo "==> staged resources/shipctl.exe for installer / resource resolve"

cd apps/desktop
export COREPACK_ENABLE=0
if [[ "$BUNDLE_INSTALLER" -eq 1 ]]; then
  echo "==> tauri build (NSIS installer)"
  ./node_modules/.bin/tauri build
else
  echo "==> tauri build --no-bundle"
  ./node_modules/.bin/tauri build --no-bundle
fi
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

if [[ "$BUNDLE_INSTALLER" -eq 1 ]]; then
  NSIS_DIR="$ROOT/target/release/bundle/nsis"
  echo "==> NSIS output:"
  ls -la "$NSIS_DIR" 2>/dev/null || ls -la "$ROOT/target/release/bundle" 2>/dev/null || true
fi
echo "Run: $DESKTOP"
