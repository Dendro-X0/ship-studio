# Self-host serve UI freeze — design

**Status:** Slice 1 shipped (L1 check) — L2 operator dogfood  
**Updated:** 2026-09-27  
**Investigation:** [selfhost-serve-ui-freeze-investigation.md](./selfhost-serve-ui-freeze-investigation.md)

```text
GOAL:     Desktop stays responsive for the full selfhost --serve session;
          Cancel serve / Esc / #btn-cancel can kill the process tree.
NOT:      Changing selfhost CLI semantics · moving serve to an external terminal
```

## Invariants

1. **Main-thread rule** — No `child.wait()` (or other unbounded block) on the Tauri main thread.  
2. **Cancel rule** — `cancel_shipctl` remains callable while a run is in flight.  
3. **API rule** — Frontend `invoke("run_shipctl")` contract unchanged (`CmdResult`).

## Slice 1 — Off-main `run_shipctl`

| Change | Detail |
|--------|--------|
| `run_shipctl` | `#[tauri::command(async)]` (sync body OK — Tauri runs it off main) |
| `run_shipctl_env` | Same |
| `cancel_shipctl` | Leave sync + short; kill tree via existing `ActiveRun.pid` |

Optional later: true `async fn` + `spawn_blocking` if we add awaits; not required for Slice 1.

## Out of scope

- Converting every short command to async  
- Changing `selfhost --serve` sleep loop  
- Opening serve in Windows Terminal (reliability class T)  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo check -p ship-studio-desktop` |
| L2 | Harbor → Self-host Deploy → click around Deployment UI for ≥10s (no Not Responding) |
| L2 | Cancel serve → process ends · dock Ready · `selfhostServing` clear |
| L2 | Esc / Output Cancel still kills mid-serve |
