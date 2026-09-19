# Launch CI + container parity — band #49

**Status:** Done (first slice)  
**Updated:** 2026-09-19  
**Parent:** cut-order-ci-registry · container-final-mile · launch companion parity · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences CI Actions check (`gh run list` read-only) and container local build + registry push docs — same honesty as Advanced Publish. Bridge never creates releases, never `docker push`. Prefer Publish for the full Adaptive path.

## Behavior

| Id | When | Open | Run |
|----|------|------|-----|
| `ci.release` | `ci_release` | Actions URL | `gh run list --workflow … --limit 5` |
| `container.build` | `container` | container docs | `docker build` or `docker compose build` |
| `container.deploy` | `container` | container docs | none (push Confirm) |

## Acceptance

- [x] release/deploy workflow → `ci.release` with `gh run list`  
- [x] Dockerfile → `container.build` docker build + `container.deploy`  
- [x] compose-only → compose build Run  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_ci_container` |
