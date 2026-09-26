# Provider wizard setup — Hosting · Payments · Email

**Status:** Slice 1 + overhaul O4 catalog diet  
**Updated:** 2026-09-25  
**Parents:** platforms-catalog · provider-portal · [ship-studio-overhaul-design](../backend/ship-studio-overhaul-design.md) O4  
**Product overview:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)

## Goal

Each Platforms / Integrations wizard must answer **what to open**, **what to finish on the vendor**, **where secrets land**, and **how to return to Publish** — in ≤5 steps. Studio never writes secrets or replaces dashboards.

## Minute loop (every wizard)

```text
Put secrets (Tier A) / Login CLI → Open dashboard when needed → Continue publishing → Confirm
  (Learn more is optional — never the setup path)
```

| Lane | Done when |
|------|-----------|
| Hosting | Live URL exists; Public Live check can Verify; or Local intent for desktop-only |
| Payments | Product/checkout live on vendor; `PUBLIC_*` or host secrets set **outside** Studio; Publish listing Confirm |
| Email | Key on deploy host via Env Put; test send on vendor UI |

## Slice 1 (this pass)

| Change | Detail |
|--------|--------|
| Cloudflare icon | `cloudflare.svg` (not Hono placeholder) |
| Hosting steps | Open → CLI/login hint → deploy → paste URL → Confirm / Live check |
| Payments steps | Named env keys (`PUBLIC_POLAR_*`, host secrets) · Confirm listing |
| Resend steps | Env Put · never paste into Studio · test on Resend |
| Handoff CTA | **Continue publishing** on Platforms + Integrations wizards |

## Non-goals

- Studio OAuth / secret storage  
- Auto-Confirm after Open  
- Fake “deployed” without URL evidence  

## Proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Open Polar / Cloudflare / Resend wizard — steps name Open → put → Publish Confirm |
