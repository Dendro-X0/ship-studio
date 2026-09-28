# Evidence — Harbor soft dogfood (S1.15–16)

**Date:** 2026-09-28  
**Subject:** `fixtures/harbor`  
**shipctl:** `target/release/shipctl.exe`  
**Commit under test:** `f24b938`

## Setup

1. `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1`
2. Bind path for Desktop: `E:\Web Projects\ship-studio\fixtures\harbor`

## L2 results (CLI)

| Check | Result |
|-------|--------|
| Scopes detect | `desktop.desktop` · `docs.website` |
| Nested parent git ignored | `pulse.git.is_repo=false` · note “Nested under … parent git status ignored” |
| Own-repo dirty (temp `git init` + untracked file) | `is_repo=true` · `dirty=true` · `dirty_count=1` |
| Folder path targets | `apps/desktop` · `apps/website` exist on disk |
| Dirty-sensitive Publish steps (Public plan) | `ship.desktop_cut` · `submit.*` · `deploy` · `live_check` · `sign.self.release*` present |
| Soft-gate predicate (mirrored) | scopes/human **off** · deploy/list/live_check/desktop_cut/release **on** |
| Scopes set / Confirm | `set --ids desktop.desktop` then both ids; `publish … confirm` advances scopes |
| Cleanup | Removed temp Harbor `.git` + `DOGFOOD_DIRTY.txt`; `harbor-reset` again |

## Desktop UI (operator — remaining)

CLI cannot click Studio chrome. After bind Harbor on Desktop built from `f24b938`:

| Check | How |
|-------|-----|
| Auto-detect Targets | Bind → Targets cards without Detect |
| Folder | Card **Folder** opens `apps/desktop` or `apps/website` |
| Deploy | Card **Deploy** opens Deployment (selfhost when no provider) |
| Dirty Save | Uncheck one target → **Save changes** cue → Save clears |
| Dirty Confirm | Temp `git init` + dirty file in Harbor → Public → Confirm on `live_check`/`deploy` → toast **Confirm anyway** · **Git status** |

## Verdict

**CLI L2 pass** for pulse nested-ignore, dirty pulse when Harbor is its own repo, scopes detect/set, and sensitive-step coverage. **Desktop UI L2** still operator-owned (Folder / Deploy / dirty Save / Confirm toast).
