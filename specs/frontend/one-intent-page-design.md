# One intent page — S1.4

**Status:** Slice 1 shipped (`pnpm website:build`)  
**Updated:** 2026-09-28  
**Parent:** [product-website-charter](../backend/product-website-charter.md) · [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · [never-say](./never-say-block-design.md)  
**Surface:** `apps/website` `/intent`

## Problem

Strangers confuse Ship Studio with:

| Lookalike | Why it fails the ICP |
|-----------|----------------------|
| CI-only “ship on green” dashboards | No Open→Confirm choreography for vendors |
| Endless checklist apps | No Adaptive Publish spine / human-gate honesty |
| Belonging / community theater | Marketing as identity, not final-mile utility |

SEO and sales need **one clear intent page** that says what this is *for*, without overselling deploy or OAuth.

## Goal

`/intent` answers in one scroll:

1. **For:** multi-surface final-mile (sign → release → deploy) across CLI · TUI · Desktop  
2. **Not for:** replacing Cloudflare/Polar/Apple · CI-only green checks · checklist theater  
3. **Honesty:** Open vendor doors · Confirm · Studio never writes secrets  

## Non-goals

- Redesigning `/` hero  
- Cafe/FAQ (S1.6) or OPERATOR-NEXT marketing (S1.5)  
- Fake comparison tables with competitor logos  
- One-click / SmartScreen claims ([never-say](./never-say-block-design.md))

## Slice 1

| Deliverable | Detail |
|-------------|--------|
| Page | `apps/website/src/pages/intent.astro` |
| Nav | Primary link **Intent** (after Pricing or before Docs) |
| Sections | (1) Who it’s for · (2) vs CI-only / checklist · (3) How honesty works · CTAs Demo/Docs/Pricing |
| Style | Existing `section` / `lead` / site tokens — no new card grid |

## Proof

| Layer | Check |
|-------|--------|
| L1 | `pnpm website:build` |
| L2 | `/intent` reachable · never-say greps clean |

## Later

- Cross-link from `/` support line  
- Structured data / OG image  
