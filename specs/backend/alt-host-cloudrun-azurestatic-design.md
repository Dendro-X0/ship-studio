# Alt host expand (Cloud Run / Azure Static) — band #46

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** alt-host-heroku-amplify (#42) · north star  
**Owner:** `publish` · `config` · `portal` · `launch`

## Product framing

Advanced + Launch Open dashboard + Confirm for Google Cloud Run and Azure Static Web Apps. No Orbit deploy.

## Acceptance

- [x] Advanced + Cloud Run markers → `host.cloudrun`  
- [x] Advanced + staticwebapp.config.json → `host.azurestatic`  
- [x] Launch includes both when detected  
- [x] General / Local omit  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl host_cloudrun` |
