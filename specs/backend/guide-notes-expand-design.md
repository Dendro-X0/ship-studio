# Guide notes expand — band #39

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** assist-notes-expand (#37) · doctor-notes-expand (#38) · north star  
**Owner:** `guide`

## Product framing

`shipctl guide` offline checklist surfaces the same portal lanes as Assist / Doctor / Pulse, and prefers **Publish** over legacy Flow.

## Behavior

- Publish guide step with Watch cue  
- Dedicated notes: alt hosts · mobile BaaS · Stripe/Paddle · Watch  
- Doctor cut-readiness notes appended from the end (not early PATH noise alone)

## Acceptance

- [x] Guide plan includes step id `publish`  
- [x] Fixture with `fly.toml` → notes mention host / Fly  
- [x] Fixture with Stripe env → notes mention Stripe / listing  
- [x] Notes mention `publish watch`  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl guide_notes` |
