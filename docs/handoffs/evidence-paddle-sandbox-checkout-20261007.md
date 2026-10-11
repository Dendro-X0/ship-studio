# Evidence — Paddle Sandbox Solo Checkout — 2026-10-07

**Status:** **Pass** (operator self · Sandbox)  
**Surface:** `https://ship-studio.pages.dev/pricing/`  
**Not:** Commercial ADMIT · Integrations product · live money

## Claim

Orbit Yard Solo Buy overlay works end-to-end in **Paddle Sandbox** when:

1. Product + price (`pri_…`) + client token (`test_…`) are in `apps/website/.env`
2. Default payment link = `https://ship-studio.pages.dev/` (localhost often rejected by dashboard)
3. Site is **rebuilt** (`pnpm website:build`) then **hostdeploy** to Cloudflare Pages (static `PUBLIC_*` bake)

## Observed

| Step | Result |
|------|--------|
| Overlay Order summary | $29 · Orbit Yard · qty 1 |
| Test card | `4242…` · Pay $29.00 |
| Bank contact | “We are contacting your bank…” |
| Success | `/checkout/success` — “You’re in.” · license / Download / Start here CTAs |

## Soft-no

- License email still **maintainer-issued** (`issue-license.py`) until webhooks exist  
- Live Paddle + approved domain = separate cut  
- Does not reopen DevOps GTM or Integrations-as-product  

## Canon

[PADDLE-SETUP](../../apps/website/docs/PADDLE-SETUP.md) · [website-paddle-checkout-spec](../frontend/website-paddle-checkout-spec.md)
