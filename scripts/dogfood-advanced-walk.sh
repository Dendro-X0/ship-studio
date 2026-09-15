#!/usr/bin/env bash
# L3 spine walk: Confirm through Advanced steps that have desktop_view Related targets.
# Does not open browsers — proves shipctl state + Related metadata Desktop relies on.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${1:?usage: $0 <project-path>}"
SHIPCTL="${SHIPCTL_PATH:-$ROOT/target/release/shipctl.exe}"
[[ -x "$SHIPCTL" ]] || SHIPCTL="$ROOT/target/debug/shipctl.exe"
[[ -x "$SHIPCTL" ]] || { echo "shipctl not found"; exit 1; }

echo "==> reset advanced · $PROJECT"
"$SHIPCTL" publish --mode advanced --project "$PROJECT" reset >/dev/null

RELATED_OK=0
RELATED_FAIL=0
STEPS_SEEN=0
while true; do
  OUT="$("$SHIPCTL" publish --mode advanced --project "$PROJECT")"
  FINISHED="$(echo "$OUT" | grep -m1 '"finished"' | grep -o 'true\|false' || true)"
  CUR_ID="$(echo "$OUT" | python -c "import sys,json; d=json.load(sys.stdin); c=d.get('current') or {}; print(c.get('id') or '')" 2>/dev/null || true)"
  VIEW="$(echo "$OUT" | python -c "import sys,json; d=json.load(sys.stdin); c=d.get('current') or {}; print(c.get('desktop_view') or '')" 2>/dev/null || true)"
  IDX="$(echo "$OUT" | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('current_index',0))" 2>/dev/null || true)"
  TOTAL="$(echo "$OUT" | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('total',0))" 2>/dev/null || true)"
  if [[ -z "$CUR_ID" ]]; then
    echo "FAIL parse current step"
    exit 1
  fi
  STEPS_SEEN=$((STEPS_SEEN + 1))
  case "$VIEW" in
    scopes|env|sign|portal|ritual|tools|dashboard|launch|"")
      echo "ok  [$IDX/$TOTAL] $CUR_ID  related=${VIEW:-none}"
      RELATED_OK=$((RELATED_OK + 1))
      ;;
    *)
      echo "BAD [$IDX/$TOTAL] $CUR_ID  related=$VIEW (Desktop RELATED_VIEW_LABELS miss)"
      RELATED_FAIL=$((RELATED_FAIL + 1))
      ;;
  esac
  if [[ "$FINISHED" == "true" ]]; then
    break
  fi
  # Cap runaway
  if [[ "$STEPS_SEEN" -gt 80 ]]; then
    echo "FAIL too many steps"
    exit 1
  fi
  "$SHIPCTL" publish --mode advanced --project "$PROJECT" confirm >/dev/null
  # Stop when confirm completed the plan (avoids double-print of last step).
  AFTER="$("$SHIPCTL" publish --mode advanced --project "$PROJECT")"
  if echo "$AFTER" | grep -m1 '"finished"' | grep -q 'true'; then
    break
  fi
done

echo "==> walked $STEPS_SEEN steps · related_ok=$RELATED_OK related_fail=$RELATED_FAIL"
[[ "$RELATED_FAIL" -eq 0 ]]
