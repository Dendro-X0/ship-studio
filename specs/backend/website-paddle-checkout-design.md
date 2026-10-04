# Studio Solo checkout — Paddle Billing overlay

**Status:** Design + overlay wired (live dogfood needs maintainer `.env`)  
**Updated:** 2026-10-03  
**Owner:** `apps/website` `commerce.ts` + `/pricing`  
**Parent:** [product-website-charter](./product-website-charter.md) · [commerce-creem-waffo-design](./commerce-creem-waffo-design.md)

## Plan alignment

- **Handoff atomic step:** Maintainer activated commerce — **Paddle** (Billing overlay), not Polar E2E.
- **PAUSED/CANCELLED:** Polar paid checkout E2E stays deferred. Vendor coach CANCELLED. No Studio-held API secrets. No creating Paddle products from `orbityard`.
- **In scope:** Env-gated Paddle.js v2 overlay on `/pricing`; honest copy; Desktop Paddle wizard Open aligned with Billing Get started (sandbox first).
- **Out of scope:** Paddle secret API keys in the repo; webhook license automation; Polar checkout buttons; replacing the Paddle dashboard.

## Two jobs (unchanged)

| Job | Owner |
|-----|--------|
| Help operators set up Paddle for **their** app | Desktop Integrations `paddle` wizard |
| Sell **Orbit Yard Solo** on this site | **Paddle overlay** (`PUBLIC_PADDLE_*`) |

Polar remains an Integrations **guide** lane. It is not the Solo checkout owner.

## Why overlay

Paddle Billing Get started (catalog → checkout → fulfillment) expects [Paddle.js overlay](https://developer.paddle.com/build/checkout/build-overlay-checkout): client-side token + price id. Card data never hits Orbit Yard servers.

## Contracts

Website env (all `PUBLIC_*`, no server secret):

| Variable | Role |
|----------|------|
| `PUBLIC_PADDLE_CLIENT_TOKEN` | `test_…` or `live_…` from Paddle → Authentication |
| `PUBLIC_PADDLE_PRICE_ID` | `pri_…` from Catalog (same env as token) |
| `PUBLIC_PADDLE_ENV` | `sandbox` (default) or `live` |
| `PUBLIC_PADDLE_PRICE_LABEL` | Display price; fallback `Solo` |
| `PUBLIC_PADDLE_PORTAL_URL` | Optional **customer** portal; empty default — never vendors.paddle.com |

`checkoutConfigured()` is true only when token **and** price id are non-empty.

```typescript
Paddle.Environment.set(env === "live" ? "production" : "sandbox");
Paddle.Initialize({ token });
Paddle.Checkout.open({
  items: [{ priceId, quantity: 1 }],
  settings: { successUrl: `${origin}/checkout/success` },
});
```

Default payment link (Paddle → Checkout → Checkout configuration) must allow the site origin (`localhost:4321` ok on **sandbox**).

## Operator minute (Paddle dashboard)

Screenshot: Billing **Get started** · Live vs Sandbox.

1. **Open sandbox** page buttons (`/products`, `/authentication`, `/checkout-settings`, `/orders`) until a test card (`4242…`) completes. Live (`vendors.paddle.com`) is a separate account and real money.
2. Catalog → product **Orbit Yard Solo** + one-time price.
3. Developer tools → Authentication → client-side token (not a server API key).
4. Checkout → Checkout settings → default payment link.
5. Put `PUBLIC_PADDLE_*` on the website host — never into Desktop.
6. Confirm `listing.paddle` when using Studio as the operator hub.

Open dashboard: `https://sandbox-vendors.paddle.com/` (portal minute). Docs stay Learn more only.

## Subtraction

- `/pricing` Buy CTA is Paddle overlay or disabled setup note — **no** “Checkout with Polar”.
- Polar env vars may remain unused; do not dual-CTA.

## Acceptance

- [x] Token+price unset → disabled Buy + pointer to `PADDLE-SETUP.md`
- [ ] Token+price set → overlay opens; success can land `/checkout/success` (L3 — maintainer `.env`, not git)
- [x] Copy says Paddle processes cards; Studio never sees PAN
- [x] Integrations Paddle wizard names sandbox-first + `PUBLIC_PADDLE_*`

## Proof

| Layer | Command |
|-------|---------|
| L1 | `pnpm website:build` |
| L3 | `pnpm website:dev` · overlay on `/pricing` after local `.env` (maintainer tokens, not git) |
