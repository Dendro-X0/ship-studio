# Commerce expand (Creem / Waffo) + Paddle wizard depth

**Status:** Design + first slice shipped  
**Updated:** 2026-10-02  
**Band / backlog:** Catalog depth (on demand) — Integrations Payments guide  
**Parent:** [commerce-stripe-paddle-design](./commerce-stripe-paddle-design.md) · [commerce-portal-catalog-design](./commerce-portal-catalog-design.md) · [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Owner:** `config` · `publish` · `secrets` · `portal` · Desktop `integrations-data.ts`

## Plan alignment

- **Handoff atomic step:** Idle queue activated by maintainer — deepen Payments/Integrations wizard dogfood (guide slice).
- **PAUSED/CANCELLED check:** Polar **Studio Solo** checkout E2E stays deferred (`payment_ready`). Vendor handoff coach **CANCELLED**. No store API upload. No Studio-held secrets.
- **In scope:** Detect + Advanced `listing.*` Open/Confirm + portal catalog + Desktop Payments wizards for **Creem** and **Waffo**; deepen **Paddle** wizard copy (still dashboard Open, not Billing API).
- **Out of scope:** Creating products/checkouts from Studio; calling vendor HTTPS with secrets; selling Ship Studio Solo via Paddle/Creem/Waffo (website Buy remains Polar).

## Product framing

Two jobs stay distinct:

| Job | Surface |
|-----|---------|
| Help the operator set up **their** checkout | Desktop Integrations → Open vendor → Put keys **outside** Studio → Confirm listing |
| Sell **Ship Studio Solo** | `apps/website` Polar env — **not** this slice |

Creem ([creem.io](https://www.creem.io/)) and Waffo Pancake ([waffo.ai](https://www.waffo.ai/)) follow Gumroad/Stripe/Paddle honesty: portal Open + Confirm only.

## Detect

| Flag | Signals |
|------|---------|
| `creem` | markets `creem`; `CREEM_` env; package `creem` / `@creem/` |
| `waffo` | markets `waffo` / `pancake`; `WAFFO_` env; package `@waffo/` |

Paddle detect unchanged (`paddle` markets · `PADDLE_` · `@paddle/`).

## Steps / URLs

| Id | Open | Docs |
|----|------|------|
| `listing.paddle` (existing) | `https://sandbox-vendors.paddle.com/products` | overlay docs (Learn more) |
| `listing.creem` | `https://creem.io/dashboard` | `https://docs.creem.io` |
| `listing.waffo` | `https://pancake.waffo.ai/merchant/auth/signin` | `https://docs.waffo.ai` |

Advanced + Public only (same as Stripe/Paddle).

## Name-only secret catalog

| Provider | Example names (never stored in `.ship/`) |
|----------|------------------------------------------|
| Creem | `CREEM_API_KEY` · `CREEM_WEBHOOK_SECRET` · `CREEM_PRODUCT_ID` |
| Waffo | `WAFFO_MERCHANT_ID` · `WAFFO_PRIVATE_KEY` · `WAFFO_WEBHOOK_SECRET` |
| Paddle | existing `PADDLE_*` + `PADDLE_CLIENT_TOKEN` (Billing overlay) |

`put_secret` on commerce providers bails to Open + put on deploy host.

## ProviderId

`creem` · `waffo` (alias `pancake`). `is_commerce()` includes Polar + Gumroad + Lemon + Stripe + Paddle + Creem + Waffo.

## Acceptance

- [x] Advanced + Creem/Waffo markets or env → `listing.creem` / `listing.waffo` with listed Open URLs
- [x] General omits those listing steps
- [x] `ProviderId::parse("creem"|"waffo"|"pancake")`
- [x] Desktop Payments catalog shows Paddle · Creem · Waffo with Open → Put outside Studio → Continue publishing
- [x] Website Buy CTA remains Polar-only (disabled until `PUBLIC_POLAR_CHECKOUT_URL`)

## Proof

| Layer | Command / action | Claim |
|-------|------------------|--------|
| L1 | `cargo test -p shipctl commerce_creem_waffo stripe_paddle commerce_portal` | implemented |
| L1 | desktop `tsc` if available; `pnpm website:build` | implemented |
| L3 | Desktop Integrations: Open Creem/Waffo/Paddle wizards — not claimed this session unless dogfood capture |

## Parallel paths

None to subtract — extend the existing commerce catalog owner (`portal` + `integrations-data`).
