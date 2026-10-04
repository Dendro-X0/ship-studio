#!/usr/bin/env bash
# Stage portable desktop pair: orbit-yard-desktop.exe + orbityard.exe side by side.
# With --installer: also build NSIS setup.exe (bundles resources/orbityard.exe).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BUNDLE_INSTALLER=0
for arg in "$@"; do
  case "$arg" in
    --installer) BUNDLE_INSTALLER=1 ;;
  esac
done

echo "==> cargo build -p orbityard --release"
cargo build -p orbityard --release

ORBITYARD_SRC="$ROOT/target/release/orbityard.exe"
if [[ ! -f "$ORBITYARD_SRC" ]]; then
  ORBITYARD_SRC="$ROOT/target/release/orbityard"
fi
mkdir -p apps/desktop/src-tauri/resources
cp -f "$ORBITYARD_SRC" apps/desktop/src-tauri/resources/orbityard.exe
echo "==> staged resources/orbityard.exe for installer / resource resolve"

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

DESKTOP="$ROOT/target/release/orbit-yard-desktop.exe"
ORBITYARD="$ROOT/target/release/orbityard.exe"
if [[ ! -f "$DESKTOP" ]]; then
  DESKTOP="$ROOT/target/release/orbit-yard-desktop"
  ORBITYARD="$ROOT/target/release/orbityard"
fi

DESKTOP_DIR="$(dirname "$DESKTOP")"
if [[ "$(cd "$(dirname "$ORBITYARD")" && pwd)/$(basename "$ORBITYARD")" != "$(cd "$DESKTOP_DIR" && pwd)/$(basename "$ORBITYARD")" ]]; then
  cp -f "$ORBITYARD" "$DESKTOP_DIR/"
else
  echo "==> orbityard already beside desktop exe"
fi
echo "==> staged:"
ls -la "$DESKTOP" "$DESKTOP_DIR/$(basename "$ORBITYARD")" 2>/dev/null || true

if [[ "$BUNDLE_INSTALLER" -eq 1 ]]; then
  NSIS_DIR="$ROOT/target/release/bundle/nsis"
  echo "==> NSIS output:"
  ls -la "$NSIS_DIR" 2>/dev/null || ls -la "$ROOT/target/release/bundle" 2>/dev/null || true
fi
echo "Run: $DESKTOP"
