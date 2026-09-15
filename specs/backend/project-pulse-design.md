# Project pulse — local status intelligence

**Status:** Active  
**Updated:** 2026-09-14  
**Owner:** `crates/shipctl` (`pulse`) · Desktop Dashboard / Now

## Problem

Dashboard shows stack chips but not whether the repo is clean, mid-publish, or already deployed. Operators need one glance + one CTA, not a robotic restart.

## Invariants

- Local signals only — `git` status (no fetch), `.ship/*`, wrangler/vercel local state files.
- No vendor HTTPS from the bridge.
- No secret values in pulse JSON.

## Signals

| Domain | Sources | Derived |
|--------|---------|---------|
| Git | `git rev-parse`, `status --porcelain`, `log -1`, `rev-list --left-right --count @{u}...HEAD` (best-effort) | branch, dirty, ahead/behind, last commit |
| Ship | `.ship/studio.json`, `launch.json`, `publish.json`, `last-run.json` | mid-wizard step, finished, last run ok |
| Deploy | `.orbit/runs/*/summary.json` (ok+url) · `.ship/last-run.json` · nested `.vercel` · `.orbit/state.json` · local `.wrangler` | `orbit_deployed` / `last_run_ok` / `vercel_linked` / `orbit_configured` / `wrangler_local` / `unknown` + `urls[]` |
| Tools | lightweight PATH probe | Signet/Orbit found |

**Live vs local:** only `orbit_deployed`, `last_run_ok`, or non-empty `urls` mean “already live”. Bare `.wrangler/state` is `wrangler_local` (dev) — does **not** auto-skip redeploy.

When live, Publish marks `deploy*` + `live_check` Done (`specs/backend/deploy-status-skip-design.md`).

## Now policy (priority)

1. No project → bind  
2. **Publish in progress** → Continue publish (current step title) — even if Orbit/Signet missing  
3. Launch in progress (no publish) → Continue launch / prefer Start publish  
4. Tools hard-block **only** when project needs Signet (Tauri / signet.toml) and Signet (or Orbit, if not already provider-linked) is missing  
5. Dirty git after a finished publish → “Commit or stash before deploy” (hint; do not auto-commit)  
6. last-run ok → “Ship again”  
7. **wrangler_state / vercel_linked** → “Already deployed via Cloudflare Workers / Vercel” (Orbit optional)  
8. Else → Start publish  

API / Worker stacks must not show **Blocked: Tools missing** solely because Orbit is off PATH when `.wrangler` (or Vercel link) already proves a prior deploy.
## Commands

```text
shipctl pulse [--project .]
```

MCP: `ship_pulse`. Desktop bind + Dashboard refresh call pulse and drive Now + health + quick actions.

## Proof

- L1: fixture with `.git` dirty + launch.json mid-step  
- L2: `shipctl pulse` JSON shape  
- L3: Desktop Now title matches current publish/launch step after bind  
