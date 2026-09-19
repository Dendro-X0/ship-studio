# Doctor notes expand — band #38

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** adaptive-doctor-verify (#15) · assist-notes-expand (#37) · north star  
**Owner:** `adapters` (doctor)

## Product framing

Doctor cut-readiness notes mention portal lanes from #28–#35 the same way Assist / Pulse do — without failing `doctor.ok` when alt-host CLIs are missing (deploy stays on vendor UI).

## Behavior

| Detected | Doctor note |
|----------|-------------|
| `fly` / `railway` / `render` / `digitalocean` | Advanced `host.*` dashboards; optional CLI on PATH is helpful, not required for ok |
| Mobile + BaaS | Advanced `baas.provision` Open console |
| `stripe` / `paddle` | Advanced `listing.*` commerce Open |
| Any layout | Cue: `publish watch` for local Verify poll |

### Provider CLI status (informational)

| Provider | Bin |
|----------|-----|
| fly | `flyctl` (also accepts `fly`) |
| railway | `railway` |
| digitalocean | `doctl` |

Missing alt-host CLIs **must not** flip `doctor.ok` false.

## Acceptance

- [x] Fixture with `fly.toml` → note mentions `host.*` / Fly  
- [x] Fixture with `STRIPE_` → note mentions Stripe / listing  
- [x] Mobile + firebase → note mentions `baas.provision`  
- [x] `doctor.ok` still true without flyctl when only `fly.toml` present  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl doctor_notes_hosts` |
