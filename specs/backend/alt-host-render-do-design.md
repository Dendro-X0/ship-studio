# Alt host portal expand (Render / DigitalOcean) — band #32

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** alt-host-portal-design (#30) · north star  
**Owner:** `publish` · `config` · `portal`

## Product framing

Same as Fly/Railway: dashboard Open + Confirm. No Orbit deploy for these hosts.

## Detect / steps

| Flag | Signals | Step | URL |
|------|---------|------|-----|
| `render` | `render.yaml` · `RENDER_` · `@render.com` package | `host.render` | `https://dashboard.render.com/` |
| `digitalocean` | `.do/app.yaml` · `DIGITALOCEAN_` / `DO_API_` · `digitalocean` package | `host.digitalocean` | `https://cloud.digitalocean.com/apps` |

Advanced + Public only; Local/General omit.

## Acceptance

- [x] Advanced fixtures get both host steps with correct URLs  
- [x] General / Local omit  
- [x] Portal providers `render` / `digitalocean`  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl host_` |
