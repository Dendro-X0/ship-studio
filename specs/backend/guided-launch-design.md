# Guided launch workflow — design

**Status:** Active  
**Updated:** 2026-09-13  
**Owner:** `crates/shipctl` (`launch`) · Desktop Launch panel

## Vision

Semi-automated shipping: for each task, **open the official entry point** → operator completes work **on the vendor platform** → tool **verifies** (CLI check or operator confirm) → **advance** to the next step → repeat until launch.

The tool never replaces Cloudflare / GitHub / Polar / Vercel; it sequences and verifies.

## State

Persisted at `.ship/launch.json` (no secrets):

```json
{
  "schema": "ship-studio/launch/v1",
  "project": "…",
  "current": 0,
  "steps": [
    { "id": "doctor", "status": "done", "verified_at": "…" },
    { "id": "paste.GITHUB_TOKEN", "status": "pending" }
  ]
}
```

Plan is rebuilt each run from project detection; statuses merge by `id`.

## Step kinds

| kind | open | verify |
|------|------|--------|
| `auto` | none / optional | local CLI (doctor, configure file, flow plan) |
| `oauth` | provider login/dashboard URL | `wrangler whoami` / `vercel whoami` / `gh auth status` |
| `paste` | value **source** URL (GitHub/Polar…) | operator `confirm` **or** `wrangler secret list` contains name (network, optional) |
| `deploy` | none | `orbit` status / last-run ok **or** confirm |

## Commands

```text
shipctl launch --project .              # show current step + progress
shipctl launch open --project .         # open entry_url for current
shipctl launch verify --project .       # run verify; print ok/fail
shipctl launch confirm --project .      # mark current human-attested done
shipctl launch next --project .         # advance if current done (or --force)
shipctl launch reset --project .        # clear statuses
```

## Desktop

**Launch** panel: progress strip · current title/detail · Open · Verify · Confirm · Next.

## Invariants

1. Bridge does not call vendor HTTPS APIs itself; verify may spawn local provider CLIs (operator-initiated).
2. Never store secret values.
3. Official platforms remain the place of work; shipctl only sequences.

## Proof

- L1: plan builds for wrangler fixture; merge status; next advances
- L2: `shipctl launch` / `verify` / `next` on assess-api offline steps (doctor/configure)
- L3: Desktop Launch panel wired
