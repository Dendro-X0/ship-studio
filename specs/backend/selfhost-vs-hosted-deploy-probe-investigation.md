# Self-host vs hosted Deploy probe — investigation

**Status:** Fixed (2026-09-27)  
**Updated:** 2026-09-27  
**Symptom:** Sign/Publish Deploy card shows **READY** + `http://127.0.0.1:…/` + “Prior evidence found” without a hosted deploy (Harbor Public intent).

## Root cause

| Layer | Behavior |
|-------|----------|
| `orbityard selfhost` | Writes `.ship/last-run.json` with `ok: true`, steps `selfhost` / `selfhost.check`, `urls: ["http://127.0.0.1:PORT/"]` |
| `pulse::deploy_pulse` | Treated any last-run `ok` as `last_run_ok`; collected loopback URLs |
| `deploy_is_live` | True for `last_run_ok` **or** any non-empty `urls` |
| Desktop probe | `deployOk` if signal live **or** any URL → Ready / “Prior evidence found” |

Self-host health is **local-auto**, not Cloudflare/Vercel/Orbit live.

## Fix

1. Detect selfhost last-run → signal `selfhost_ok` (not `last_run_ok`).  
2. `deploy_is_live` false for `selfhost_ok` and loopback-only URL sets.  
3. Desktop: **Self-host** badge / pill; hosted Ready only via `deployEvidenceFromPulse`.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p orbityard -- selfhost_last_run_is_not_hosted_live` (+ `loopback_only_urls_are_not_live`, `orbit_summary_marks_live`) |
| L1 | Orbit summary still live |
| L2 | Harbor after selfhost → Deploy badge **Self-host**, not Ready |

## Non-goals

- Removing selfhost last-run  
- Auto-clearing last-run on intent change  
