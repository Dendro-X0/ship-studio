# OPERATOR-NEXT as product feature — S1.5

**Status:** Slice 1 shipped (`pnpm website:build`)  
**Updated:** 2026-09-28  
**Parent:** [OPERATOR-NEXT](../../docs/product/OPERATOR-NEXT.md) · [human-gate-catalog-design](../backend/human-gate-catalog-design.md) · [never-say](./never-say-block-design.md) · [one-intent-page](./one-intent-page-design.md)  
**Surface:** `apps/website` `/honesty` · `/docs/human-gates`

## Problem

Strangers read human gates (Paste · Sign · Listing · Deploy Confirm) as **unfinished bugs** or “the app isn’t done.”  
Maintainer truth: **human gates are the product** — Open → Confirm choreography is the honesty layer competitors fake with auto-green.

## Goal

Market OPERATOR-NEXT as a **feature**:

1. Public page that leads with honesty framing (not a bash dump)  
2. Docs entry copy that says “gates you attest,” not “remaining work”  
3. OPERATOR-NEXT itself opens with product framing; lane table ≠ backlog

## Non-goals

- Auto-Confirm OAuth / store / DNS  
- Replacing `/docs/human-gates` raw content with marketing only  
- Cafe/FAQ (S1.6)  
- Never-say violations

## Slice 1

| Deliverable | Detail |
|-------------|--------|
| `/honesty` | Marketing page: gates = product · status layers · refuse auto-green · CTAs |
| Nav | Link **Honesty** (near Intent) |
| Docs index | Lead + human-gates card reframe |
| `OPERATOR-NEXT.md` | Product kicker; rename “Remaining human work” → “Human gates by lane” |
| Cross-links | `/intent` · `/docs/human-gates` · `/demo` |

## Proof

| Layer | Check |
|-------|--------|
| L1 | `pnpm website:build` |
| L2 | `/honesty` + `/docs/human-gates` readable · never-say clean |

## Later

- Pull human-gate catalog examples onto `/honesty`  
- Desktop Dashboard blurb linking honesty  
