# Deploy status & skip redundant deploy

**Status:** Implemented  
**Owner:** `crates/shipctl` (`pulse`, `publish`, `config`) · Desktop checklist

## Problem

Dashboard could say “already deployed” from a bare `.wrangler` folder (often local miniflare only), while Publish always scheduled `deploy*` + `live_check`. Operators re-ran Cloudflare deploys unnecessarily.

## Signals (local only — no vendor HTTPS)

| Signal | Evidence | Drives “live”? |
|--------|----------|----------------|
| `orbit_deployed` | Latest `.orbit/runs/*/summary.json` with `ok: true` (+ `url` / `apiUrl` / `docsUrl`) | **Yes** |
| `last_run_ok` | `.ship/last-run.json` successful deploy step or overall ok | **Yes** |
| `vercel_linked` | `.vercel/project.json` at root or `apps/*/` | Soft (linked) |
| `orbit_configured` | `.orbit/state.json` provider `configured: true` | Soft |
| `wrangler_local` | `.wrangler/state` or `tmp` only (no successful orbit summary) | **No** |
| `unknown` | None | No |

`DeployPulse.urls` populated from orbit summaries and optional `last-run.json` `urls` / `url`.

## Publish skip policy

When `deploy_is_live`:

- On `load_or_build`, mark matching `deploy*` **and** `live_check` as `Done` with detail `skipped — already live (…)`.
- Set `live_check.entry_url` to the first known URL when present.
- Operator can still **Open/Run** on a redeploy pass after `publish reset` or by confirming a fresh plan after reset.
- Mid-wizard Confirm progress for other steps is preserved; only unsatisfied deploy/live_check get auto-done when live.

## Persistence

- Successful `shipctl deploy` writes `.ship/last-run.json` (including URLs from latest orbit summary when available).

## Proof

- L1: fixture with only `.wrangler/state` → `wrangler_local`, not live  
- L1: fixture with orbit summary ok + url → `orbit_deployed`, General publish marks deploy + live_check Done  
- L1: `cargo test -p shipctl pulse:: publish::`  
- L3: assess-api Dashboard shows workers.dev / Deployed; Publish skips redeploy steps  
