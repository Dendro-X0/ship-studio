# Pulse cut hints expand — band #36

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** pulse-cut-hints (#17) · north star  
**Owner:** `pulse`

## Product framing

Dashboard Now mid-publish detail cues the new portal lanes (hosts, BaaS, marketplace submit, Stripe/Paddle) the same way it cues CI / npm.

## Scope

Expand `publish_cut_hint` for `oauth.*`, `db.provision`, `baas.provision`, `host.*`, `submit.*`, and commerce/store `listing.*` families.

Keep Continue as primary; hints are detail suffixes only.

## Acceptance

- [x] Unit asserts for `host.fly`, `baas.provision`, `submit.steam`, `listing.stripe`  
- [x] Empty for unrelated ids still empty  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl publish_cut_hints` |
