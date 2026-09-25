# Provider wizard setup — Hosting · Payments · Email

**Status:** Slice 1 shipped (working tree) — remaining gaps listed in product overview  
**Updated:** 2026-09-25  
**Parents:** platforms-catalog · provider-portal · S1.10 / S1.13 backlog  
**Product overview:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)

## Goal

Each Platforms / Integrations wizard must answer **what to open**, **what to finish on the vendor**, **where secrets land**, and **how to return to Publish** — in ≤5 steps. Studio never writes secrets or replaces dashboards.

## Minute loop (every wizard)

```text
Open dashboard → finish on vendor → put env on host (Env / secrets put)
  → Back to Publish / Continue publishing → Confirm (or Live check URL)
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
