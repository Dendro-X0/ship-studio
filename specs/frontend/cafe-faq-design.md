# Cafe / FAQ stub — S1.6

**Status:** Slice 1 shipped (`pnpm website:build`)  
**Updated:** 2026-09-28  
**Parent:** [never-say](./never-say-block-design.md) · [one-intent](./one-intent-page-design.md) · [operator-next-product](./operator-next-product-design.md) · pricing  
**Surface:** `apps/website` `/faq`

## Problem

Strangers hit the same four questions before buying or cloning:

| Question | Risk if unanswered |
|----------|-------------------|
| SmartScreen? | Expect Studio to silence OS trust UI (never-say) |
| Other Ship Studio / ship.studio? | Brand collision with unrelated agentic product |
| Why $29? | Thinks OSS build is incomplete vs Solo packaging |
| Signet required? | Blocks on tool install when Local/docs-only cut doesn’t need it |

## Goal

A short FAQ (“cafe”) page with honest answers — stub depth, not a knowledge base.

## Non-goals

- Full support portal / search  
- Claiming SmartScreen silence or verified publisher  
- Expanding into S2.9 Pro tier pitch  

## Slice 1

| Deliverable | Detail |
|-------------|--------|
| `/faq` | Four Q&As + CTAs to Intent / Honesty / Pricing / Docs |
| Nav | **FAQ** link |
| Cross-links | Pricing note · home optional |

## Proof

| Layer | Check |
|-------|--------|
| L1 | `pnpm website:build` |
| L2 | Never-say greps clean on `/faq` |

## Later

- More Qs from real support mail  
- Anchor IDs for deep links from Desktop toasts  
