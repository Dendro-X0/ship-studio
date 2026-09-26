# Current session — Ship Studio

**Updated:** 2026-09-25  
**Branch:** `main`  
**Status:** Reliability slice 1 (Env Put terminal) — committing with this handoff

## Next Atomic Step

**Implement [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md) slice 2** — unify `open_shipctl_terminal` + expand `isSoftCmdFailure`; migrate existing openers. Or park until next session.

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
| **Reliability slice 1** | Env Put → `open_env_put_terminal` — design + investigation |
| Platforms · Portal · hosting parity | [46384c4](https://github.com/Dendro-X0/ship-studio/commit/46384c4) |

## Boot allowlist

1. This file  
2. [desktop-reliability-design.md](../../specs/backend/desktop-reliability-design.md)  
3. `apps/desktop/src-tauri/src/lib.rs` · `main.ts`  
4. `pnpm exec tsc --noEmit` · `cargo check -p ship-studio-desktop`  
