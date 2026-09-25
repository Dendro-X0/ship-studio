# Platforms catalog — Hosting + Official signing

**Status:** Slice 1 + hosting parity 0–3 (working tree)  
**Updated:** 2026-09-25  
**Parents:** status-probe · public-publish-ux · integrations wizards  
**Product overview:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Hosting tiers:** [hosting-portal-parity-design](../backend/hosting-portal-parity-design.md)

## Problem

Deploy and Official signing dead-end into Sign paths or “Use Local” without a **provider list** like Payments / Email. Operators cannot pick a host or store lane from one place, so utility UI (cards · sidebar tree · checklist · Open dashboard) is duplicated or missing.

## North star

```text
Inspection Deploy / Official CTA
  → Platforms catalog (same chrome as Integrations)
  → pick Hosting or Official signing provider
  → checklist + Open / Docs / Portal steps (when honest) → Continue publishing
```

One **provider catalog** utility powers Integrations and Platforms.

## Slice 1 (shipped)

| Surface | Behavior |
|---------|----------|
| View | `platforms` — catalog + wizard aside (reuse `.int-*` chrome) |
| Sidebar | **Platforms** tree: Hosting · Official signing (mirror Payments · Email) |
| Deploy CTA | Probe **Choose host** → Platforms (Hosting) |
| Official CTA | Probe **Choose platform** → Platforms (Official signing) |
| Select | Open vendor URL; optional Portal filter / ritual `deploy_args` hint |
| Shared util | `provider-catalog.ts` — catalog grid · sidebar rows · wizard paint |

### Hosting (honest) — Desktop catalog = Tiers A+B+C+E

| Tier | Lanes |
|------|-------|
| A | Cloudflare · Vercel · Netlify — Portal Login CLI |
| B | Fly · Railway — Portal Login CLI |
| C | GitHub Pages — Open GitHub + Docs only (no PAT portal) |
| E | Orbit — no portal id |

**Tier D** (Render · DO · Heroku · Amplify · Cloud Run · Azure SWA): Advanced Publish `host.*` + `shipctl portal --provider` only — not Platforms cards yet.

### Official signing (honest)

Apple · Microsoft · Google Play — guide only; Confirm stays on Publish after vendor work.

## Non-goals

- Studio OAuth to hosts or stores  
- Fake “one-click deploy”  
- Replacing Ritual advanced presets  
- Auto-Confirm after Open  
- Expanding Tier D into Platforms until a dedicated catalog slice  

## Proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Publish probe → Choose host → Platforms list; Official → same view Signing group |
| L2 | GitHub Pages → Docs visible; Portal steps hidden |

## Later

| Slice | Change |
|-------|--------|
| Reliability design | Command/toast/terminal parity — not drive-by; see [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md) gaps |
| Detect → highlight | Pulse lights matching Platforms card (hosting parity slice 4) — **done** |
| Outcome panel | Local vs Public success copy after finish |
| Catalog mount | Migrate Integrations HTML onto shared catalog helpers only |
| Tier D Platforms cards | Only when product asks to expand the catalog |
