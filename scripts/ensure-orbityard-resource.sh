#!/usr/bin/env bash
# Ensure apps/desktop/src-tauri/resources/orbityard.exe exists for `tauri dev` / build.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/apps/desktop/src-tauri/resources/orbityard.exe"
mkdir -p "$(dirname "$DEST")"

pick() {
  for c in \
    "$ROOT/target/debug/orbityard.exe" \
    "$ROOT/target/release/orbityard.exe" \
    "$ROOT/target/debug/orbityard" \
    "$ROOT/target/release/orbityard"
  do
    if [[ -f "$c" ]]; then
      echo "$c"
      return 0
    fi
  done
  return 1
}

SRC="$(pick || true)"
if [[ -z "${SRC:-}" ]]; then
  echo "==> cargo build -p orbityard (for Tauri resources)"
  (cd "$ROOT" && cargo build -p orbityard)
  SRC="$(pick)"
fi

if [[ ! -f "$DEST" ]] || [[ "$SRC" -nt "$DEST" ]]; then
  cp -f "$SRC" "$DEST"
  echo "==> staged $DEST"
fi
