# Current session — Ship Studio

**Updated:** 2026-09-25  
**Branch:** `main`  
**Status:** Reliability **slices 0–4 closed** (L1) — Later bands parked

## Next Atomic Step

**Park reliability** until maintainer picks a Later band (non-Windows terminals · CDP attach · vault UX) or a new design.

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| Polar paid checkout E2E | Deferred |
| aperio Advanced L4 | Parked |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |
| Drive-by Desktop reliability | Work only via design slices |
| Reliability Later | Non-Windows terminals · CDP · vault UX — separate activation |

## Last closed

| Band | Link |
|------|------|
| **Reliability slice 4** | Advanced Deploy/Flow → terminal; dry-run stays J (this commit) |
| **Reliability slice 3** | [925f1a9](https://github.com/Dendro-X0/ship-studio/commit/925f1a9) — user load honesty |
| **Reliability slice 2** | [15be2f4](https://github.com/Dendro-X0/ship-studio/commit/15be2f4) — unified terminal opener |
| **Reliability slice 1** | [406eed9](https://github.com/Dendro-X0/ship-studio/commit/406eed9) — Env Put terminal |

## Boot allowlist

1. This file  
2. [desktop-reliability-design.md](../../specs/backend/desktop-reliability-design.md)  
3. `apps/desktop/src/main.ts` · `apps/desktop/index.html`  
4. `pnpm exec tsc --noEmit` · `cargo check -p ship-studio-desktop`  
