# Current session — Ship Studio

**Updated:** 2026-09-25  
**Branch:** `main`  
**Status:** Reliability slices 1–2 on main (after this commit)

## Next Atomic Step

**Implement [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md) slice 3** — user-initiated Load env / Refresh toast on fail. Or park until next session.

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
| **Reliability slice 2** | `open_shipctl_terminal` · soft-fail taxonomy (this commit) |
| **Reliability slice 1** | [406eed9](https://github.com/Dendro-X0/ship-studio/commit/406eed9) — Env Put terminal |

## Boot allowlist

1. This file  
2. [desktop-reliability-design.md](../../specs/backend/desktop-reliability-design.md)  
3. `pnpm exec tsc --noEmit` · `cargo check -p ship-studio-desktop`  
