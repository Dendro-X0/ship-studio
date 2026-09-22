# Polar portal Open → always Overview

**Status:** Implement  
**Owner:** `crates/shipctl/src/portal.rs`  
**Surface:** Desktop Portal · `shipctl portal`

## Problem

Polar has no CLI OAuth, so the plan emits `dashboard` · `token_page` · `env`. All three set `entry_url` to `https://polar.sh/dashboard` (`token_url` == `create_url`, and `env` reused `token_url`). Open therefore always lands on Polar Overview.

Polar’s in-app routes need an org slug (`/dashboard/{slug}/products`, `/settings`, `/settings/webhooks`). Without a slug, `/dashboard/products` is treated as an org name and 404s.

## Fix

| Step | Without slug (docs / entry) | With local `POLAR_ORGANIZATION_SLUG` |
|------|-----------------------------|-------------------------------------|
| dashboard | `https://polar.sh/dashboard` | `…/dashboard/{slug}/products` |
| token_page | `https://polar.sh/docs/integrate/oat` | `…/dashboard/{slug}/settings` |
| env | `https://polar.sh/docs/integrate/webhooks/endpoints` | `…/dashboard/{slug}/settings/webhooks` |

Slug is read only from project `.env` / `.env.local` / `.dev.vars` (and one-level app dirs) — no Polar HTTPS.

## Proof

- `cargo test -p shipctl polar_dashboard_steps`
- `shipctl portal --project . --provider polar --json` → three distinct `entry_url`s
