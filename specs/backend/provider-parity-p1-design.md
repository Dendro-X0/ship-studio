# Provider parity P1 — Tier A/B vs Cloudflare dogfood

**Status:** Shipped P1.1–P1.5  

**Updated:** 2026-10-07  
**Parents:** [host-automation-dual-path](./host-automation-dual-path-design.md) · [host-xor polish](./host-xor-value-ux-polish-design.md) · [PLATFORMS](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Out of scope:** Tier D logos (Render/DO/…) · Integrations product · DevOps GTM

## Plan alignment

```text
HANDOFF ATOMIC STEP: Better support for other providers = CF dogfood parity on Tier A/B
PAUSED / CANCELLED:  Tier D catalog sprawl · Integrations expansion
CANONICAL OWNER:     hostdeploy + Desktop Deploy toast / results bay
PROOF:               L1 URL fallback tests · L2 Vercel/Netlify Deploy shows Open live
```

## Gap (from CF dogfood)

Cloudflare now: create-if-missing · `--host-name` · live URL toast · Troubleshoot bay.  
Peers lag on URL certainty, Vercel project name, Fly/Railway recovery UI.

## Slice P1 (this band)

| # | Change | Done when |
|---|--------|-----------|
| P1.1 | Vercel Deploy dialog name → `vercel deploy --project <name>` | Name not readOnly; hostdeploy `--name` wired |
| P1.2 | Success URL fallback when CLI omits https | Netlify `*.netlify.app` · Fly `*.fly.dev` · Vercel `*.vercel.app` (last resort) · CF Pages already |
| P1.3 | Fly `project_name` from `fly.toml` `app =` | Fallback URL matches app |
| P1.4 | Desktop: recognize fly.dev / railway.app as hosted URLs | Open live works |
| P1.5 | Fly/Railway results bay + Troubleshoot/Retry | Same recovery surface as Tier A |

## Never

- Auto `fly launch` / `railway link` (human `[link]` stays)
- Fake success without exit 0
- Add Render/Heroku/… Platforms cards

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apply_project_name` vercel adds `--project`; fallback URL unit tests |
| L2 | Deploy Vercel/Netlify → toast Open live; Fly fail → Troubleshoot visible |
