#!/usr/bin/env bash
# Stage portable desktop + orbityard sidecar for the current OS.
# With --installer: tauri build with platform bundles (NSIS / DMG / deb·AppImage).
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

ORBITYARD_SRC="$ROOT/target/release/orbityard"
if [[ -f "$ROOT/target/release/orbityard.exe" ]]; then
  ORBITYARD_SRC="$ROOT/target/release/orbityard.exe"
fi
if [[ ! -f "$ORBITYARD_SRC" ]]; then
  echo "error: orbityard release binary not found" >&2
  exit 1
fi

RES="$ROOT/apps/desktop/src-tauri/resources"
mkdir -p "$RES"
# Single sidecar — `.exe` suffix keeps Windows CreateProcess happy; Unix still execs by path.
cp -f "$ORBITYARD_SRC" "$RES/orbityard-bin.exe"
echo "==> staged resources/orbityard-bin.exe for Tauri resources"

cd apps/desktop
export COREPACK_ENABLE=0
if [[ ! -x ./node_modules/.bin/tauri ]]; then
  echo "==> pnpm install (desktop)"
  pnpm install
fi
if [[ "$BUNDLE_INSTALLER" -eq 1 ]]; then
  echo "==> tauri build (platform installers)"
  ./node_modules/.bin/tauri build
else
  echo "==> tauri build --no-bundle"
  ./node_modules/.bin/tauri build --no-bundle
fi
cd "$ROOT"

DESKTOP="$ROOT/target/release/orbit-yard-desktop"
ORBITYARD="$ROOT/target/release/orbityard"
if [[ -f "$ROOT/target/release/orbit-yard-desktop.exe" ]]; then
  DESKTOP="$ROOT/target/release/orbit-yard-desktop.exe"
  ORBITYARD="$ROOT/target/release/orbityard.exe"
fi

DESKTOP_DIR="$(dirname "$DESKTOP")"
SIDE_NAME="$(basename "$ORBITYARD")"
if [[ "$(cd "$(dirname "$ORBITYARD")" && pwd)/$SIDE_NAME" != "$(cd "$DESKTOP_DIR" && pwd)/$SIDE_NAME" ]]; then
  cp -f "$ORBITYARD" "$DESKTOP_DIR/"
else
  echo "==> orbityard already beside desktop binary"
fi
echo "==> staged:"
ls -la "$DESKTOP" "$DESKTOP_DIR/$SIDE_NAME" 2>/dev/null || true

if [[ "$BUNDLE_INSTALLER" -eq 1 ]]; then
  BUNDLE="$ROOT/target/release/bundle"
  echo "==> bundle output:"
  ls -la "$BUNDLE" 2>/dev/null || true
  for sub in nsis dmg macos deb appimage rpm msi; do
    if [[ -d "$BUNDLE/$sub" ]]; then
      echo "---- $sub ----"
      ls -la "$BUNDLE/$sub" || true
    fi
  done
fi
echo "Run: $DESKTOP"
