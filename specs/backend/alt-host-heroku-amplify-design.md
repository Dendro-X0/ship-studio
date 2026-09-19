# Alt host expand (Heroku / Amplify) — band #42

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** alt-host-portal (#30) · alt-host-render-do (#32) · north star  
**Owner:** `publish` · `config` · `portal`

## Product framing

Same portal honesty as Fly/Railway/Render/DO: Advanced + Public Open dashboard + Confirm. No Orbit deploy.

## Detect / steps

| Flag | Signals | Step | URL |
|------|---------|------|-----|
| `heroku` | `heroku.yml` · `.heroku/` · `HEROKU_` · package · markets · Procfile+`app.json` | `host.heroku` | dashboard.heroku.com/apps |
| `amplify` | `amplify/` · `amplify.yml` · config files · `AMPLIFY_` · `@aws-amplify` · markets | `host.amplify` | console.aws.amazon.com/amplify |

Portal `ProviderId::{Heroku, Amplify}`. Local/General omit.

## Acceptance

- [x] Advanced + heroku.yml → `host.heroku`  
- [x] Advanced + amplify.yml → `host.amplify`  
- [x] General / Local omit  
- [x] Portal providers include both when detected  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl host_heroku` |
