# Commerce portal catalog — band #40

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** graduate-commerce · commerce expand (#35) · north star  
**Owner:** `portal` · `secrets`

## Product framing

Gumroad / Lemon / Stripe / Paddle / Creem / Waffo are first-class portal providers beside Polar — dashboard Open only; no Payment Link / SKU creation from the bridge.

## Behavior

| ProviderId | Detect | Dashboard |
|------------|--------|-----------|
| Gumroad | `gumroad` | app.gumroad.com |
| Lemon | `lemon` | app.lemonsqueezy.com |
| Stripe | `stripe` | dashboard.stripe.com |
| Paddle | `paddle` | sandbox-vendors.paddle.com |
| Creem | `creem` | creem.io/dashboard |
| Waffo | `waffo` (`pancake`) | pancake.waffo.ai |

`is_commerce()` covers Polar + these — `put_secret` bails to Open + put on deploy host.

## Acceptance

- [x] Stripe env → portal includes `stripe`  
- [x] Gumroad markets → portal includes `gumroad`  
- [x] `ProviderId::parse("paddle"|"lemonsqueezy")`  
- [x] `put_secret` Stripe bails  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl commerce_portal` |
