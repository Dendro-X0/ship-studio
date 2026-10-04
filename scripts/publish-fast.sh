#!/usr/bin/env bash
# Fast path: burn Auto/ready publish gates (stops at Human/Open).
# Usage: bash scripts/publish-fast.sh [project] [--mode general|advanced] [--intent local|public] [--chain N]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${1:-.}"
shift || true
MODE="general"
INTENT="local"
CHAIN="20"
EXTRA=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --intent) INTENT="$2"; shift 2 ;;
    --chain) CHAIN="$2"; shift 2 ;;
    *) EXTRA+=("$1"); shift ;;
  esac
done

ORBITYARD="${ORBITYARD_PATH:-}"
if [[ -z "$ORBITYARD" ]]; then
  if [[ -x "$ROOT/target/debug/orbityard.exe" ]]; then ORBITYARD="$ROOT/target/debug/orbityard.exe"
  elif [[ -x "$ROOT/target/debug/orbityard" ]]; then ORBITYARD="$ROOT/target/debug/orbityard"
  elif [[ -x "$ROOT/target/release/orbityard.exe" ]]; then ORBITYARD="$ROOT/target/release/orbityard.exe"
  elif [[ -x "$ROOT/target/release/orbityard" ]]; then ORBITYARD="$ROOT/target/release/orbityard"
  else
    (cd "$ROOT" && cargo build -p orbityard)
    ORBITYARD="$ROOT/target/debug/orbityard"
    [[ -x "${ORBITYARD}.exe" ]] && ORBITYARD="${ORBITYARD}.exe"
  fi
fi

exec "$ORBITYARD" publish --mode "$MODE" --intent "$INTENT" --project "$PROJECT" continue --chain "$CHAIN" "${EXTRA[@]}"
