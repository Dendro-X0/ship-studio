# Demo subject — Harbor (lightweight)

**Status:** Implement  
**Updated:** 2026-09-24  
**Parent:** [docs/assets/demo/v0.2.1/SCRIPT.md](../../docs/assets/demo/v0.2.1/SCRIPT.md)

## Problem

Public demos fail when recorded on **ship-studio** (already signed/deployed) or on **advanced-dogfood** (API + Mobile heavy — wrong Targets story for Part 1 GIFs).

## Decision

| Subject | Role |
|---------|------|
| **`fixtures/harbor`** | **Canonical demo + GIF shelf** — Desktop + Docs (+ Signet), mid-flight friendly |
| `fixtures/advanced-dogfood` | Advanced multipath dogfood only (CI · mobile · container · steam …) |
| Ship Studio monorepo | Engineering dogfood — **not** public Part 1 GIFs |
| aperio / personal apps | Forbidden on public clips |

Harbor is a **stage set**, not a product to sell.

## Layout (minimal)

```text
fixtures/harbor/
  README.md
  signet.toml                 → Signet probe
  apps/desktop/src-tauri/     → Desktop scope (tauri)
  apps/website/package.json   → Docs scope
  apps/website/index.html     → camera face
```

No wrangler · android · Dockerfile · markets — keeps Targets to **Desktop · Docs**.

## Demo discipline

1. Bind `fixtures/harbor` (or run `scripts/harbor-reset`).  
2. Wipe `.ship/` before recording so Scopes / Configure / Sign·Deploy start unfinished.  
3. **Advanced + Local** for the spine; **Public** only for a short Open beat.  
4. Window ~960×560 → GIF **720×420**; Output dock hidden.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `shipctl scopes --project fixtures/harbor` → `desktop.desktop` · `docs.website` |
| L2 | Desktop bind → Targets Desktop + Docs; probe not all-Ready after reset |
| L3 | T1–T4 GIFs use Harbor path only |

## Non-goals

- Fake live Polar / store upload  
- Marketing site for Harbor as a real product  
- Replacing Advanced dogfood scripts  
