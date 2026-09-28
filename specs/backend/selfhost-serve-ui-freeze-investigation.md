# Self-host serve UI freeze — investigation

**Status:** Fixed (Slice 1)  
**Updated:** 2026-09-27  
**Symptom:** Desktop title bar **Ship Studio (Not Responding)** within seconds of **Deployment → Self-host → Deploy** (`shipctl selfhost --serve`). Cancel serve / Cancel / UI clicks dead. Status may still show `RUNNING · Ns` from a timer started before the hang.

## Evidence

| Observation | Meaning |
|-------------|---------|
| Output: `$ … shipctl.exe selfhost --serve` | Long-lived child started |
| Results: `health:200` + loopback URL while still RUNNING | `write_run` completed; serve loop held the process |
| Windows **(Not Responding)** | Native UI message pump stuck |
| Cancel serve visible but useless once hung | Cancel IPC cannot run while main thread is blocked |

## Root cause

`apps/desktop/src-tauri/src/lib.rs` — `run_shipctl` / `run_shipctl_env` are **synchronous** `#[tauri::command]`s that call `child.wait()`.

Tauri 2: sync commands run on the **main thread** unless marked `async` / `#[tauri::command(async)]` ([docs](https://v2.tauri.app/develop/calling-rust/)).

`shipctl selfhost --serve` intentionally never exits until killed (`crates/shipctl/src/selfhost.rs` sleep loop). Therefore:

1. Main thread blocks in `wait()` for the whole serve session.  
2. WebView / window chrome stop pumping → **Not Responding**.  
3. `cancel_shipctl` cannot run on the same thread → Cancel appears dead.

Any long `run_shipctl` (not only selfhost) risks the same freeze; `--serve` makes it inevitable.

## Non-causes

- Frontend `setBusy` / 1s tick (JS continues briefly until OS marks hung)  
- `emit` flood after serve starts (almost no further lines)  
- shipctl health check itself (completes; freeze is post-check wait)

## Fix direction

1. Run `run_shipctl` / `run_shipctl_env` off the main thread (`#[tauri::command(async)]` or `async` + `spawn_blocking`).  
2. Keep `cancel_shipctl` short + sync (or async) so it can kill the tree while wait runs elsewhere.  
3. Proof: Harbor Self-host Deploy → UI stays responsive → Cancel serve ends process → Ready.

## Related

- [desktop-reliability-design](./desktop-reliability-design.md) — Cancel unlock invariant; this bug violates it for serve.  
- Handoff PAUSED “drive-by reliability” — this is a **named** freeze slice, not drive-by nav polish.
