#!/usr/bin/env bash
# Dogfood Advanced publish steps for Desktop Related matrix.
# Usage: from ship-studio root — bash scripts/dogfood-advanced-publish.sh
# Writes/refreshes fixtures/advanced-dogfood and asserts the Advanced plan.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIX="${1:-$ROOT/fixtures/advanced-dogfood}"
mkdir -p "$FIX/.ship" "$FIX/.github/workflows" "$FIX/android" "$FIX/ios"
printf '%s\n' 'name = "dogfood"' '[[d1_databases]]' 'binding = "DB"' 'database_name = "x"' 'database_id = "…"' >"$FIX/wrangler.toml"
printf '%s\n' '{"expo":{"name":"dogfood","slug":"dogfood"}}' >"$FIX/app.json"
printf '%s\n' '// stub' >"$FIX/android/build.gradle"
printf '%s\n' 'FROM alpine' >"$FIX/Dockerfile"
printf '%s\n' 'name: release' 'on: push' >"$FIX/.github/workflows/release.yml"
printf '%s\n' '480' >"$FIX/steam_appid.txt"
printf '%s\n' '["itch","epic","npm","crates","marketing","graduate","gumroad","lemon"]' >"$FIX/.ship/markets.json"
mkdir -p "$FIX/apps/website"
printf '%s\n' '<!doctype html><title>dogfood</title>' >"$FIX/apps/website/index.html"
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
    git -C "$FIX" remote add origin "https://github.com/example/ship-studio-dogfood.git" 2>/dev/null || true
  fi
fi

SHIPCTL="${SHIPCTL_PATH:-}"
EXPLICIT_SHIPCTL=0
if [[ -n "$SHIPCTL" ]]; then
  EXPLICIT_SHIPCTL=1
fi
if [[ -z "$SHIPCTL" ]]; then
  # Prefer freshly built debug over stale release (Desktop may stage release separately).
  if [[ -x "$ROOT/target/debug/shipctl.exe" ]]; then
    SHIPCTL="$ROOT/target/debug/shipctl.exe"
  elif [[ -x "$ROOT/target/debug/shipctl" ]]; then
    SHIPCTL="$ROOT/target/debug/shipctl"
  elif [[ -x "$ROOT/target/release/shipctl.exe" ]]; then
    SHIPCTL="$ROOT/target/release/shipctl.exe"
  elif [[ -x "$ROOT/target/release/shipctl" ]]; then
    SHIPCTL="$ROOT/target/release/shipctl"
  else
    (cd "$ROOT" && cargo build -p shipctl)
    SHIPCTL="$ROOT/target/debug/shipctl"
    [[ -x "${SHIPCTL}.exe" ]] && SHIPCTL="${SHIPCTL}.exe"
  fi
fi

# If we auto-picked release and it's older than sources, rebuild debug and use it.
if [[ "$EXPLICIT_SHIPCTL" -eq 0 && "$SHIPCTL" == *"/release/"* ]]; then
  (cd "$ROOT" && cargo build -p shipctl >/dev/null)
  if [[ -x "$ROOT/target/debug/shipctl.exe" ]]; then
    SHIPCTL="$ROOT/target/debug/shipctl.exe"
  elif [[ -x "$ROOT/target/debug/shipctl" ]]; then
    SHIPCTL="$ROOT/target/debug/shipctl"
  fi
fi

echo "Fixture: $FIX"
echo "shipctl: $SHIPCTL"
OUT="$("$SHIPCTL" publish --mode advanced --project "$FIX")"
NEED=(
  listing.play
  listing.app_store
  listing.steam
  listing.itch
  listing.epic
  listing.npm
  listing.crates
  submit.play
  submit.app_store
  db.provision
  ci.release
  container.deploy
  legal.baseline
  trust.pack
  marketing.deploy
  sign.graduate
  listing.gumroad
  listing.lemon
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
for pair in "listing.steam:portal" "listing.npm:portal" "listing.crates:portal" "listing.gumroad:portal" "listing.lemon:portal" "db.provision:env" "ci.release:dashboard" "container.deploy:portal" "submit.play:sign" "legal.baseline:dashboard" "trust.pack:sign" "sign.graduate:sign" "marketing.deploy:portal"; do
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
# Keep a current shipctl beside debug desktop for live dogfood (mtime-aware resolve).
if [[ -x "$ROOT/target/debug/ship-studio-desktop.exe" || -x "$ROOT/target/debug/ship-studio-desktop" ]]; then
  if [[ -x "$SHIPCTL" ]]; then
    cp -f "$SHIPCTL" "$ROOT/target/debug/$(basename "$SHIPCTL")" 2>/dev/null || true
    echo "Staged $(basename "$SHIPCTL") next to target/debug desktop."
  fi
fi
