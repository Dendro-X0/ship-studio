#!/usr/bin/env bash
# Reset Harbor demo fixture to a clean mid-flight state (no .ship session).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HARBOR="$ROOT/fixtures/harbor"
if [[ ! -d "$HARBOR" ]]; then
  echo "missing $HARBOR" >&2
  exit 1
fi
rm -rf "$HARBOR/.ship"
echo "Harbor reset: removed $HARBOR/.ship"
echo "Bind in Desktop: $HARBOR"
