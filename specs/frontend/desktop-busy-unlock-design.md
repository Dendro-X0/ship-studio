# Desktop busy / Refresh deadlock — band #25

**Status:** Done (first slice, 2026-09-19)  
**Parent:** release-surface value backlog · aperio L3 dogfood  
**Owner:** `apps/desktop` (`main.ts`)

## Problem

While `running` is true, Publish **Refresh** and most actions stay disabled. If Cancel did not clear the frontend lock (or `setBusy(false)` ran without `setProjectUi`), the UI stayed on **RUNNING…** with Refresh permanently disabled.

## Fix

1. `setBusy` always re-syncs project button disabled state via `setProjectUi`.  
2. `run()` clears busy in `finally` (success, fail, or throw).  
3. **Cancel** force-unlocks the UI even when Rust reports nothing to kill.  
4. Optional busy watchdog toast after ~90s reminding the operator to Cancel.

## Proof

- Manual: start a long `shipctl` → Cancel → Refresh enabled again.  
- Manual: Clear output while busy path → Ready + buttons restored.  
- `npx tsc --noEmit` in `apps/desktop`  
