# Publish progress watch — band #27

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** SCOPE-OF-SERVICE · publish-portal-design · north star  
**Owner:** `publish` · Desktop Publish view  

## Product framing (maintainer)

Ship Studio remains a **portal**, not a vendor replacement:

- Guides operators to the **official** surface for APIs, DBs, mobile backends, signing, store/Steam listing & submit  
- Human still completes OAuth, review, DNS, live publish  
- Bridge never calls vendor HTTPS; never store-API uploads  

**Capability:** after Open/Run, Studio can **watch locally** and prompt when Verify would succeed — so the operator is nudged to Confirm → Next without staring at the UI.

## Already covered (do not rebuild)

| Area | Adaptive steps (Advanced + Public) |
|------|-------------------------------------|
| APIs / host backends | oauth* · env · deploy* · live_check |
| Databases | `db.provision` + portal create URLs (Neon/Supabase/D1/Turso) |
| Self / official sign | `sign.self.*` · `sign.official.*` · `sign.graduate` · submit.* |
| Stores / Steam | `listing.play|app_store|steam|…` · `submit.play|app_store|microsoft` |

Gaps for later bands (still portal-only): mobile BaaS providers, Steam **submit**/depot docs step, more hosts (Fly/Railway) as Open URLs.

## Watch behavior

```text
Operator: Open/Run (vendor or local CLI)
Studio:   poll verify_current on an interval (local only)
When ok:  notify + optionally auto-Confirm (CLI --auto-confirm only)
Operator: Confirm → Next
```

### CLI

```text
shipctl publish watch [--mode …] [--intent …] [--project …]
  [--interval-secs 15] [--once] [--auto-confirm]
```

- Default: print JSON lines `{ "ok", "message", "step_id", "prompt" }` when state changes  
- `--once`: single verify (scripting); always exits 0  
- `--auto-confirm`: only when verify ok (explicit opt-in; still never live-publishes)  

### Desktop

- Publish toolbar **Watch** toggle  
- While on: poll every 15s via `shipctl publish watch --once` (skips while another command is busy)  
- On transition to verify-ok: toast “Step ready — Confirm” + Confirm button emphasis  
- Off by default; Local/Public unchanged  

### Invariants

- No vendor HTTPS from the bridge  
- No silent Confirm unless `--auto-confirm`  
- Watch stops when publish finished or Watch toggled off  
- Cancel / busy unlock (band #25) still applies to Runs; Watch uses short invokes  

## Acceptance (first slice)

- [x] `shipctl publish watch --once` returns verify result for current step  
- [x] Unit: watch once on sticky `legal.baseline` after LICENSE+SECURITY  
- [x] Desktop: Watch toggle polls and toasts (L3 manual OK)  

## Non-goals (this band)

- OS push notifications / tray daemon as a separate service  
- Watching remote store review status via vendor APIs  
- Replacing Pulse (Pulse stays on-demand Now)  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl publish::tests::watch_once` |
| L2 | `shipctl publish watch --once --project <subject>` |
| L3 | Desktop Watch on Publish (manual) |
