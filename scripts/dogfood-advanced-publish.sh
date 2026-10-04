#!/usr/bin/env bash
# Dogfood Advanced publish steps for Desktop Related matrix.
# Usage: from ship-studio root — bash scripts/dogfood-advanced-publish.sh
# Writes/refreshes fixtures/advanced-dogfood and asserts the Advanced plan.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIX="${1:-$ROOT/fixtures/advanced-dogfood}"
mkdir -p "$FIX/.ship" "$FIX/.github/workflows" "$FIX/android" "$FIX/ios" "$FIX/public"
printf '%s\n' 'name = "dogfood"' '[[d1_databases]]' 'binding = "DB"' 'database_name = "x"' 'database_id = "…"' >"$FIX/wrangler.toml"
printf '%s\n' '{"expo":{"name":"dogfood","slug":"dogfood"}}' >"$FIX/app.json"
printf '%s\n' '// stub' >"$FIX/android/build.gradle"
printf '%s\n' 'FROM alpine' >"$FIX/Dockerfile"
printf '%s\n' 'name: release' 'on: push' >"$FIX/.github/workflows/release.yml"
printf '%s\n' '480' >"$FIX/steam_appid.txt"
printf '%s\n' '["itch","epic","npm","crates","marketing","graduate","gumroad","lemon","suite","hf"]' >"$FIX/.ship/markets.json"
printf '%s\n' '{"name":"dogfood","short_name":"dogfood","display":"standalone","start_url":"/"}' >"$FIX/public/manifest.webmanifest"
printf '%s\n' '# Model card stub' >"$FIX/modelcard.md"
mkdir -p "$FIX/apps/website"
printf '%s\n' '<!doctype html><title>dogfood</title>' >"$FIX/apps/website/index.html"
printf '%s\n' '{"canonical_hint":"https://example.com/dogfood","siblings":[{"label":"sibling","path":"../sibling","env_keys":["NEXT_PUBLIC_DOGFOOD_URL"]}]}' >"$FIX/.ship/suite.json"
printf '%s\n' 'NEON_DATABASE_URL=' >"$FIX/.env.local"
# Gap #7: leave LICENSE/SECURITY/TRUST absent; add Signet marker for trust.pack.
printf '%s\n' 'name = "dogfood"' >"$FIX/signet.toml"
rm -f "$FIX/LICENSE" "$FIX/LICENSE.md" "$FIX/SECURITY.md" "$FIX/TRUST.md" "$FIX/CHANGELOG.md"
rm -f "$FIX/.ship/publish.json" "$FIX/.ship/scopes.json" "$FIX/.ship/studio.json"
# Optional GH Release step when origin is GitHub (non-Signet-self path still uses release.github).
if command -v git >/dev/null 2>&1; then
  if [[ ! -d "$FIX/.git" ]]; then
    git -C "$FIX" init -q || true
  fi
  if ! git -C "$FIX" remote get-url origin >/dev/null 2>&1; then
    git -C "$FIX" remote add origin "https://github.com/example/orbit-yard-dogfood.git" 2>/dev/null || true
  fi
fi

ORBITYARD="${ORBITYARD_PATH:-}"
EXPLICIT_ORBITYARD=0
if [[ -n "$ORBITYARD" ]]; then
  EXPLICIT_ORBITYARD=1
fi
if [[ -z "$ORBITYARD" ]]; then
  # Prefer freshly built debug over stale release (Desktop may stage release separately).
  if [[ -x "$ROOT/target/debug/orbityard.exe" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard.exe"
  elif [[ -x "$ROOT/target/debug/orbityard" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard"
  elif [[ -x "$ROOT/target/release/orbityard.exe" ]]; then
    ORBITYARD="$ROOT/target/release/orbityard.exe"
  elif [[ -x "$ROOT/target/release/orbityard" ]]; then
    ORBITYARD="$ROOT/target/release/orbityard"
  else
    (cd "$ROOT" && cargo build -p orbityard)
    ORBITYARD="$ROOT/target/debug/orbityard"
    [[ -x "${ORBITYARD}.exe" ]] && ORBITYARD="${ORBITYARD}.exe"
  fi
fi

# If we auto-picked release and it's older than sources, rebuild debug and use it.
if [[ "$EXPLICIT_ORBITYARD" -eq 0 && "$ORBITYARD" == *"/release/"* ]]; then
  (cd "$ROOT" && cargo build -p orbityard >/dev/null)
  if [[ -x "$ROOT/target/debug/orbityard.exe" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard.exe"
  elif [[ -x "$ROOT/target/debug/orbityard" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard"
  fi
fi

# Ensure debug binary matches sources (cargo test does not always refresh orbityard.exe).
if [[ "$EXPLICIT_ORBITYARD" -eq 0 ]]; then
  (cd "$ROOT" && cargo build -p orbityard >/dev/null)
  if [[ -x "$ROOT/target/debug/orbityard.exe" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard.exe"
  elif [[ -x "$ROOT/target/debug/orbityard" ]]; then
    ORBITYARD="$ROOT/target/debug/orbityard"
  fi
fi

echo "Fixture: $FIX"
echo "orbityard: $ORBITYARD"
OUT="$("$ORBITYARD" publish --mode advanced --project "$FIX")"
NEED=(
  listing.play
  listing.app_store
  listing.steam
  listing.itch
  listing.epic
  listing.npm
  listing.crates
  listing.huggingface
  submit.play
  submit.app_store
  db.provision
  ci.release
  container.build
  container.deploy
  legal.baseline
  trust.pack
  marketing.deploy
  sign.graduate
  listing.gumroad
  listing.lemon
  suite.url_sync
)
MISS=0
for id in "${NEED[@]}"; do
  if echo "$OUT" | grep -q "\"id\": \"$id\""; then
    echo "ok  $id"
  else
    echo "MISS $id"
    MISS=1
  fi
done
# Related desktop_view samples
for pair in "listing.steam:portal" "listing.npm:portal" "listing.crates:portal" "listing.huggingface:portal" "listing.gumroad:portal" "listing.lemon:portal" "db.provision:env" "ci.release:dashboard" "container.build:portal" "container.deploy:portal" "submit.play:sign" "legal.baseline:dashboard" "trust.pack:sign" "sign.graduate:sign" "marketing.deploy:portal" "suite.url_sync:dashboard"; do
  id="${pair%%:*}"
  view="${pair##*:}"
  if echo "$OUT" | tr '\n' ' ' | grep -q "\"id\": \"$id\".*\"desktop_view\": \"$view\""; then
    echo "ok  $id → $view"
  else
    # JSON pretty-print may break single-line grep; use python/jq-free check
    if echo "$OUT" | grep -A6 "\"id\": \"$id\"" | grep -q "\"desktop_view\": \"$view\""; then
      echo "ok  $id → $view"
    else
      echo "MISS $id → $view"
      MISS=1
    fi
  fi
done
if [[ "$MISS" -ne 0 ]]; then
  echo "Dogfood plan incomplete."
  exit 1
fi
echo "Advanced dogfood plan OK — bind this folder in Desktop (Advanced mode):"
echo "  $FIX"
# Keep a current orbityard beside debug desktop for live dogfood (mtime-aware resolve).
if [[ -x "$ROOT/target/debug/orbit-yard-desktop.exe" || -x "$ROOT/target/debug/orbit-yard-desktop" ]]; then
  if [[ -x "$ORBITYARD" ]]; then
    cp -f "$ORBITYARD" "$ROOT/target/debug/$(basename "$ORBITYARD")" 2>/dev/null || true
    echo "Staged $(basename "$ORBITYARD") next to target/debug desktop."
  fi
fi
