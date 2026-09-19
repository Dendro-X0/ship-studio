# Launch marketplace submit parity — band #44

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** launch-commerce-parity (#41) · marketplace-submit (#29/#31) · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences Steam / itch / Epic listing + submit Open + Confirm — same honesty as Advanced Publish. Prefer Publish for the full Adaptive path.

## Acceptance

- [x] steam_appid → listing.steam + submit.steam  
- [x] markets itch+epic → matching listing + submit  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_market` |
