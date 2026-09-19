# Launch host / BaaS parity — band #43

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** launch-commerce-parity (#41) · alt-host bands · mobile-baas (#28) · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences alt-host dashboards and mobile BaaS provision beside commerce listings — same Open + Confirm honesty as Advanced Publish. Prefer Publish for the full Adaptive path.

## Acceptance

- [x] fly.toml → `host.fly`  
- [x] mobile + firebase → `baas.provision`  
- [x] heroku.yml → `host.heroku`  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_host` |
