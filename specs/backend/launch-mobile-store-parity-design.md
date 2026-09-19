# Launch mobile / store submit parity — band #45

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** launch-marketplace-submit (#44) · store-submission-split · desktop-tauri-store-lanes (#23) · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences Play / App Store / Microsoft Store listing + submit Open + Confirm. No store API upload. Tauri-only skips Play (band #23 honesty). Prefer Publish for the full Adaptive path.

## Acceptance

- [x] android → listing.play + submit.play  
- [x] Tauri → submit.app_store + submit.microsoft; no Play  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_store` |
