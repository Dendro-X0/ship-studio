# Commerce expand (Stripe / Paddle) — band #35

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** graduate-commerce · north star  
**Owner:** `config` · `publish` · `secrets` · `portal`

## Product framing

Portal Open + Confirm for Stripe / Paddle SKUs — same honesty as Gumroad / Lemon. Bridge never creates Payment Links or calls vendor APIs.

## Detect

| Flag | Signals |
|------|---------|
| `stripe` | markets `stripe`; `STRIPE_` env; package `stripe` / `@stripe/` |
| `paddle` | markets `paddle`; `PADDLE_` env; package `@paddle/` |

## Steps

| Id | URL |
|----|-----|
| `listing.stripe` | `https://dashboard.stripe.com/` |
| `listing.paddle` | `https://vendors.paddle.com/` |

Advanced + Public only.

## Also

- `entry_url_for_secret` for `STRIPE_` / `PADDLE_`  
- Name-only secret catalog rows (like Gumroad)  

## Acceptance

- [x] Advanced + STRIPE_ / markets → `listing.stripe`  
- [x] Advanced + paddle markets → `listing.paddle`  
- [x] General omits  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl stripe_paddle` |
