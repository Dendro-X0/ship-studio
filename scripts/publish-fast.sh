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

SHIPCTL="${SHIPCTL_PATH:-}"
if [[ -z "$SHIPCTL" ]]; then
  if [[ -x "$ROOT/target/debug/shipctl.exe" ]]; then SHIPCTL="$ROOT/target/debug/shipctl.exe"
  elif [[ -x "$ROOT/target/debug/shipctl" ]]; then SHIPCTL="$ROOT/target/debug/shipctl"
  elif [[ -x "$ROOT/target/release/shipctl.exe" ]]; then SHIPCTL="$ROOT/target/release/shipctl.exe"
  elif [[ -x "$ROOT/target/release/shipctl" ]]; then SHIPCTL="$ROOT/target/release/shipctl"
  else
    (cd "$ROOT" && cargo build -p shipctl)
    SHIPCTL="$ROOT/target/debug/shipctl"
    [[ -x "${SHIPCTL}.exe" ]] && SHIPCTL="${SHIPCTL}.exe"
  fi
fi

exec "$SHIPCTL" publish --mode "$MODE" --intent "$INTENT" --project "$PROJECT" continue --chain "$CHAIN" "${EXTRA[@]}"
