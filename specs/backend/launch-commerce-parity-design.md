# Launch commerce parity — band #41

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** commerce expand (#35) · commerce portal catalog (#40) · guided-launch · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences Gumroad / Lemon / Stripe / Paddle listing Open + Confirm beside Polar — same honesty as Advanced Publish. Prefer Publish for the full Adaptive path.

## Acceptance

- [x] Stripe env → `listing.stripe`  
- [x] markets gumroad/lemon/paddle → matching listing steps  
- [x] Polar still present when detected  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_commerce` |
