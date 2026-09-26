# Current session — Ship Studio

**Updated:** 2026-09-25  
**Branch:** `main`  
**Status:** Reliability **slice 3 closed** (L1) — next: slice 4 Ritual N-class

## Next Atomic Step

**Implement [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md) slice 4** — Deploy/Flow as class N (interactive terminal when Advanced + not Offline) or honest auth-fail copy. L1: desktop `tsc` · cargo check.

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| Polar paid checkout E2E | Deferred |
| aperio Advanced L4 | Parked |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |
| Drive-by Desktop reliability | Work only via design slices |

## Last closed

| Band | Link |
|------|------|
| **Reliability slice 3** | `loadJsonCmd({ user })` toasts for Env / Assist / Scopes user paths (this commit) |
| **Reliability slice 2** | [15be2f4](https://github.com/Dendro-X0/ship-studio/commit/15be2f4) — unified terminal opener |
| **Reliability slice 1** | [406eed9](https://github.com/Dendro-X0/ship-studio/commit/406eed9) — Env Put terminal |

## Boot allowlist

1. This file  
2. [desktop-reliability-design.md](../../specs/backend/desktop-reliability-design.md)  
3. `apps/desktop/src/main.ts`  
4. `pnpm exec tsc --noEmit` · `cargo check -p ship-studio-desktop`  
