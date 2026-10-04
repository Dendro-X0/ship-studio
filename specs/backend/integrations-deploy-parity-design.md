# Integrations — Deploy-feel parity (Payments · Email)

**Status:** Design — implement this slice  
**Updated:** 2026-10-03  
**Owner:** Desktop Integrations wizards · `integrations-data.ts` · `main.ts` Put/Open  
**Parent:** [human-gate-catalog-design](./human-gate-catalog-design.md) · [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)

## Plan alignment

- **Handoff:** Maintainer wants Payments/Email as easy as Deployment after auth.
- **PAUSED:** Paddle Solo API keys / overlay dogfood — do not chase `pdl_sdbx` or website Buy.
- **CANCELLED:** Vendor handoff coach — no form-fill theater.
- **In scope:** Same *shape* as Deployment — short steps, exact Open page buttons, Put when a host secret exists, Confirm gate, Continue publishing.
- **Out of scope:** Creating checkout products via Studio; Studio-held payment API keys; one-click “Deploy” for commerce (no local CLI equivalent).

## Why Deploy feels easy

| After auth | Deployment (Cloudflare) | Payments / Email |
|------------|-------------------------|------------------|
| Local CLI Studio can stream | `wrangler` / `vercel` / … | **None** for catalog create |
| Put secret on host | Put secrets (primary) | Put only when a deploy-host secret exists (e.g. Resend) |
| Open exact console | Open dashboard | Exact page Opens (Paddle already; extend to all lanes) |
| Attest | Confirm gate | Confirm `listing.*` / `env.sprint` |

Honest ceiling: Payments stay **Open → finish on vendor → Put env outside Studio → Confirm**. Studio never becomes the MoR dashboard.

## Contracts

1. Every Payments/Email wizard has `openLinks[]` of **dashboard pages** (not docs) for the copy/paste minute.
2. Action row order matches Deployment intent: **Put** (when visible) primary · **Open** · Portal steps · Learn more (tertiary) · Confirm · Continue.
3. Resend keeps Put `RESEND_API_KEY` as primary (already).
4. Polar / Stripe / Gumroad / Lemon / Creem / Waffo: page Opens + Confirm; no fake Deploy button.
5. Paddle keeps sandbox page Opens (already); Solo website keys stay paused.

## Acceptance

- [x] All Payments + Resend wizards expose `openLinks` (exact vendor URLs)
- [x] Integrations action row: Put primary when shown; Open not Advanced-gated (already)
- [x] Copy says no product creation / no Studio-held cards
- [x] No Solo Paddle key dogfood in this slice

## Proof

| Layer | Command |
|-------|---------|
| L1 | `pnpm exec tsc --noEmit` in `apps/desktop` |
| L3 | Desktop · Integrations → each lane · Open page button lands vendor (maintainer click) |
