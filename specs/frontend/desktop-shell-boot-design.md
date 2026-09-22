# Desktop shell boot — never show Edge connection refused

**Status:** Implement  
**Owner:** `apps/desktop/src-tauri` (+ embedded `boot/shell.html`)  
**Parent:** Desktop native feel · frameless shell

## Problem

Frameless Tauri (`decorations: false`) loads `devUrl` `http://localhost:1420`. When Vite is down (crash, race, `cargo run` without CLI wait), WebView2 paints **Microsoft Edge** `ERR_CONNECTION_REFUSED`. That page has no Ship titlebar/window controls — the shell feels dead and non-native.

## Invariants

1. Operator never sees Edge / Chromium network error chrome inside the window.
2. While waiting or failed, the same custom titlebar (drag · min · max · close) stays usable.
3. Production (`frontendDist`) is unchanged — boot guard is **debug-only**.
4. When Vite returns, navigate to `http://localhost:1420/` without requiring a full app restart.

## Design

| Piece | Behavior |
|-------|----------|
| Window start | `visible: false` until Ready decides content |
| Protocol `shipboot` | Serves embedded branded `boot/shell.html` |
| Ready (debug) | If port 1420 up → show Vite URL; else navigate `shipboot` → show → wait TCP → navigate Vite |
| Watchdog (2s) | On Vite URL but TCP down → `shipboot`; on boot + TCP up → Vite |
| Boot UI | Brand · status line · Retry · same emerald charcoal as shell |

Windows custom-protocol origin: `http://shipboot.localhost/` (Tauri/WebView2). Other platforms: `shipboot://localhost/`.

## Proof

| Layer | Check |
|-------|-------|
| L1 | `cargo check -p ship-studio-desktop` |
| L2 | Open boot URL / kill Vite → branded waiting page, not Edge |
| L3 | Start Vite again → auto-navigate to app without restart |

## Out of scope

- M2 modularization of `main.ts`
- Changing production asset protocol
- Fixing Vite itself
