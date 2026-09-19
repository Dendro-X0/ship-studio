# Alt host portal (Fly / Railway) — band #30

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** SCOPE-OF-SERVICE · north star  
**Owner:** `publish` · `config` · `portal`

## Product framing

Portal Open URLs for hosts Orbit does not drive yet. Human deploys on Fly / Railway dashboards (or their CLIs). Bridge never calls vendor HTTPS; no Orbit `deploy --provider fly` in this slice.

## Detect

| Flag | Signals |
|------|---------|
| `fly` | `fly.toml` · `.fly/` · `FLY_` env · package mentions `fly.io` / `@flydotio` |
| `railway` | `railway.toml` · `railway.json` · `.railway/` · `RAILWAY_` env · package `railway` / `@railway/cli` |

## Steps

| Id | When | URL |
|----|------|-----|
| `host.fly` | Advanced + Public + `fly` | `https://fly.io/dashboard` |
| `host.railway` | Advanced + Public + `railway` | `https://railway.app/dashboard` |

Kind: Human · `desktop_view: portal` · Open + Confirm.

## Portal

`ProviderId::{Fly, Railway}` — empty `oauth_cli` (dashboard Open). No forced secrets queue.

## Filters

- Not General  
- Local omits (`host.fly` / `host.railway`)  

## Non-goals

- Orbit/Fly/Railway deploy Runs from the bridge  
- Watching remote deploy status via vendor APIs  

## Acceptance

- [x] Advanced + `fly.toml` → `host.fly` with Fly dashboard URL  
- [x] Advanced + `railway.toml` → `host.railway`  
- [x] General / Local omit  
- [x] Unit tests green  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl host_` |
