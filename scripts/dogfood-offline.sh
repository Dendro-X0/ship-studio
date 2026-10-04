#!/usr/bin/env bash
# Offline dogfood — runs everything orbityard can do without human tokens/deploy.
# Stops with a clear MANUAL checklist (see docs/product/OPERATOR-NEXT.md).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${1:-$ROOT}"
ORBITYARD="${ORBITYARD_PATH:-$ROOT/target/release/orbityard}"
if [[ -f "$ORBITYARD.exe" ]]; then ORBITYARD="$ORBITYARD.exe"; fi
if [[ ! -f "$ORBITYARD" ]]; then
  echo "==> building orbityard --release"
  (cd "$ROOT" && cargo build -p orbityard --release)
  ORBITYARD="$ROOT/target/release/orbityard"
  [[ -f "$ORBITYARD.exe" ]] && ORBITYARD="$ORBITYARD.exe"
fi

echo "==> doctor  $PROJECT"
"$ORBITYARD" doctor --project "$PROJECT" >/tmp/ship-dogfood-doctor.json
echo "    ok=$(python -c "import json;print(json.load(open('/tmp/ship-dogfood-doctor.json'))['ok'])" 2>/dev/null || echo '?')"

echo "==> guide"
"$ORBITYARD" guide --project "$PROJECT" >/tmp/ship-dogfood-guide.json

echo "==> secrets (deduped hints)"
"$ORBITYARD" secrets --project "$PROJECT" >/tmp/ship-dogfood-secrets.json

echo "==> ship (offline prep)"
"$ORBITYARD" ship --project "$PROJECT" >/tmp/ship-dogfood-ship.json

echo "==> flow --dry-run --offline --skip-deploy"
"$ORBITYARD" flow --project "$PROJECT" --dry-run --offline --skip-deploy >/tmp/ship-dogfood-flow.json

echo
echo "=== OFFLINE DONE ==="
echo "JSON under /tmp/ship-dogfood-*.json"
echo
echo "=== MANUAL NEXT (agent cannot finish) ==="
echo "1. gh auth login"
echo "2. Paste Worker secrets:  $ORBITYARD human --project \"$PROJECT\" --put"
echo "3. Optional vault:        $ORBITYARD vault export --out ship-secrets.km --from-hints --project \"$PROJECT\""
echo "4. Deploy when ready:     $ORBITYARD deploy --project \"$PROJECT\""
echo "5. Create GitHub remote + push for ship-studio (if publishing)"
echo
echo "Full checklist: $ROOT/docs/product/OPERATOR-NEXT.md"
