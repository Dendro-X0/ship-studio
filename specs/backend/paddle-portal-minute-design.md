# Paddle portal minute — sandbox-first Open

**Status:** Design + implement this slice  
**Updated:** 2026-10-03  
**Owner:** Desktop Integrations `paddle` + `orbityard` portal/publish/secrets URLs  
**Parent:** [human-gate-catalog-design](./human-gate-catalog-design.md) · [website-paddle-checkout-design](./website-paddle-checkout-design.md)

## Plan alignment

- **Handoff:** Maintainer is new to Paddle and needs Studio as the portal, not Paddle’s tutorial trail.
- **CANCELLED:** Vendor handoff coach — no form-fill, no overlay theater.
- **In scope:** Exact Open (sandbox dashboard) + Do-X menu names + env names. Sandbox is a **separate account**.
- **Out of scope:** Creating Paddle products via API; storing `pdl_…` keys; replacing the Paddle UI.

## Value

Docs-as-setup is not the product. Primary CTA **Open Paddle sandbox**. Learn more stays tertiary.

## Contracts

Verified sandbox dashboard paths (302 → login, not 404):

| Button | URL |
|--------|-----|
| Create sandbox account | `https://sandbox-vendors.paddle.com/signup` |
| Catalog (`pri_…`) | `https://sandbox-vendors.paddle.com/products` |
| Client token (`test_…`) | `https://sandbox-vendors.paddle.com/authentication` |
| Checkout settings | `https://sandbox-vendors.paddle.com/checkout-settings` |
| Orders / refunds | `https://sandbox-vendors.paddle.com/orders` (Paddle has no `/transactions` route) |
| Sandbox home (primary Open) | `https://sandbox-vendors.paddle.com/` |
| Learn more | overlay docs URL |

Live dashboard is never the first Open. Studio does not fill forms.

Env names for overlay: `PUBLIC_PADDLE_CLIENT_TOKEN` (`test_…`) · `PUBLIC_PADDLE_PRICE_ID` (`pri_…`). Do not hint `PADDLE_API_KEY` as the setup path.

## Acceptance

- [x] Paddle wizard Open lands on sandbox dashboard, not live vendors + not docs
- [x] Steps say sandbox is a separate signup
- [x] `listing.paddle` / portal / secrets entry URLs match sandbox dashboard
- [x] Learn more remains `#int-docs`, not the primary button
- [x] Wizard `openLinks` deep-link Catalog, Authentication, Checkout settings, Orders
- [x] Open vendor URLs work in General (not blocked on Advanced)
