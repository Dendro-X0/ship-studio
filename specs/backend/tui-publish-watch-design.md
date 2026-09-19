# TUI publish watch — band #33

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** publish-progress-watch-design (#27) · north star  
**Owner:** `tui` · `publish`

## Product framing

Complete Watch surface parity: CLI · Desktop · **TUI**. Same local Verify poller; never vendor HTTPS; never auto-Confirm.

## Behavior

- On Publish screen, **`w`** toggles watch (off by default)  
- While on: every ~15s call `verify_current` (no alt-screen leave)  
- On transition to ok: log `WATCH READY` + status `READY — c confirm · n next`  
- Stops when publish finished or toggled off  
- Footer / title show Watch on/off  

## Acceptance

- [x] `w` toggles `publish_watch` on Publish screen  
- [x] Idle poll path probes verify when Watch on  
- [x] Footer / status mention Watch  

## Non-goals

- Auto-confirm in TUI  
- OS notifications  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo check -p shipctl` (TUI compiles) |
| L3 | Manual: TUI Publish → w → see READY after fixing a sticky step |
