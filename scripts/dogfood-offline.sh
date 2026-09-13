#!/usr/bin/env bash
# Offline dogfood — runs everything shipctl can do without human tokens/deploy.
# Stops with a clear MANUAL checklist (see docs/OPERATOR-NEXT.md).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${1:-$ROOT}"
SHIPCTL="${SHIPCTL_PATH:-$ROOT/target/release/shipctl}"
if [[ -f "$SHIPCTL.exe" ]]; then SHIPCTL="$SHIPCTL.exe"; fi
if [[ ! -f "$SHIPCTL" ]]; then
  echo "==> building shipctl --release"
  (cd "$ROOT" && cargo build -p shipctl --release)
  SHIPCTL="$ROOT/target/release/shipctl"
  [[ -f "$SHIPCTL.exe" ]] && SHIPCTL="$SHIPCTL.exe"
fi

echo "==> doctor  $PROJECT"
"$SHIPCTL" doctor --project "$PROJECT" >/tmp/ship-dogfood-doctor.json
echo "    ok=$(python -c "import json;print(json.load(open('/tmp/ship-dogfood-doctor.json'))['ok'])" 2>/dev/null || echo '?')"

echo "==> guide"
"$SHIPCTL" guide --project "$PROJECT" >/tmp/ship-dogfood-guide.json

echo "==> secrets (deduped hints)"
"$SHIPCTL" secrets --project "$PROJECT" >/tmp/ship-dogfood-secrets.json

echo "==> ship (offline prep)"
"$SHIPCTL" ship --project "$PROJECT" >/tmp/ship-dogfood-ship.json

echo "==> flow --dry-run --offline --skip-deploy"
"$SHIPCTL" flow --project "$PROJECT" --dry-run --offline --skip-deploy >/tmp/ship-dogfood-flow.json

echo
echo "=== OFFLINE DONE ==="
echo "JSON under /tmp/ship-dogfood-*.json"
echo
echo "=== MANUAL NEXT (agent cannot finish) ==="
echo "1. gh auth login"
echo "2. Paste Worker secrets:  $SHIPCTL human --project \"$PROJECT\" --put"
echo "3. Optional vault:        $SHIPCTL vault export --out ship-secrets.km --from-hints --project \"$PROJECT\""
echo "4. Deploy when ready:     $SHIPCTL deploy --project \"$PROJECT\""
echo "5. Create GitHub remote + push for ship-studio (if publishing)"
echo
echo "Full checklist: $ROOT/docs/OPERATOR-NEXT.md"
