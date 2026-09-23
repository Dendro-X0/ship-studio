# CI — shipctl test failure (doctor on monorepo)

**Job:** `CI / shipctl test (push)`  
**Step:** CLI smoke — `shipctl doctor --project .`  
**Class:** workflow / doctor gate (layout mismatch)  
**Not:** unit test failure · lockfile · compile

## Evidence

Doctor JSON: `ok: false`, `tauri: true`, `signet_toml: false`, `signet.found: false`.  
Exit: `bail!("doctor found problems")` in `main.rs`.  
Runner has no Signet on PATH (expected for ubuntu CI without installing Signet).

## Root cause

Smoke runs Doctor against the **Ship Studio monorepo**, which probes `apps/desktop` as Tauri → `wants_signet` → fails without Signet. That is correct for shipping *a* Tauri product; it is the wrong gate for **tooling CI**.

Dead-code warning on `load_or_build_with_mode` is noise only (not the exit 1).

## Fix (one class)

**Doctor:** hard-require Signet only when `signet.toml` exists — Tauri without Signet init stays a note (`ok` can still be true). That matches CI (hub monorepo has Tauri apps/desktop, no signet.toml) and early projects before `signet init`.

**Optional:** smoke Doctor on `fixtures/ci-smoke` in CI (needs `workflow` scope to push `.github/workflows/ci.yml`).

## Proof

- `shipctl doctor --project .` on hub → `ok: true` without Signet when no `signet.toml`
- `cargo test -p shipctl --locked`
- CI `shipctl test` green
