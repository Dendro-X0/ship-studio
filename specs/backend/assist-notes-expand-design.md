# Assist notes expand — band #37

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** assist-run-notes (#19) · pulse-cut-hints-expand (#36) · north star  
**Owner:** `assist`

## Product framing

Assist checklist notes mention portal lanes from #28–#35 and Watch surfaces — without a second wizard.

## Acceptance

- [x] Fixture with `fly.toml` note mentions hosts  
- [x] Fixture with `STRIPE_` note mentions Stripe listing  
- [x] Markets steam note mentions submit  
- [x] Publish step detail mentions Watch  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl assist_notes_hosts` |
