# Solo commerce without Paddle (maintainer halt)

**Status:** Decision recorded — **no more Paddle Solo dogfood**  
**Updated:** 2026-10-04  
**Owner:** website `/pricing` + `apps/website/src/lib/commerce.ts` (when a processor is chosen)  
**Parent:** [website-paddle-checkout-design](./website-paddle-checkout-design.md) · [payments-setup-path-optimization-design](./payments-setup-path-optimization-design.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Idle — Publish orientation shipped; commerce optional
PAUSED / CANCELLED:  Portal/Integrations expansion; vendor coach; Polar E2E was deferred
CANONICAL OWNER:     website commerce (Solo Buy) vs Integrations (guide only)
PROOF BEFORE DONE:   n/a this slice — docs/handoff only
```

Maintainer: Paddle Billing is **too much hassle**; no further time on learning that stack. Official “recommended” articles do not override that.

## Decision

| Surface | Rule |
|---------|------|
| **Orbit Yard Solo Buy** | **Not Paddle.** Overlay dogfood **PAUSED**. Do not seed, Put `PUBLIC_PADDLE_*`, or Confirm live Paddle. |
| **Integrations Paddle card** | Stays a **guide** for operators who already chose Paddle. Do not expand it. |
| **OSS product** | Download / GitHub Release is enough. Paid checkout is **optional**, not the sequencer. |

## Why Paddle lost (for *this* site)

Paddle is a merchant-of-record with **account split (sandbox vs live), KYC, website approval, default payment link, client-side token ≠ API key, catalog `pri_`**. Overlay fails until those gates exist. MCP prompt packs add a second Get Started. That is lock-in-by-process, not a Studio bug. Fighting it with more Orbit Yard Opens is worse than using a vendor that sells a **single checkout URL**.

## Alternatives ranked for *our* Buy button (least learning)

Assume: one-time digital download, optional license email later, human still creates vendor keys.

| Rank | Option | Operator minute | Fit |
|------|--------|-----------------|-----|
| **0** | **No Buy** — GitHub Releases only | Zero | Matches current motivation. Site already works. |
| **1** | **Polar checkout link** | Product → Checkout link → paste `PUBLIC_POLAR_CHECKOUT_URL` | Already documented in `POLAR-SETUP.md`. Studio used this before Paddle overlay. No default payment link, no `test_`/`pri_` pair. |
| **2** | **Gumroad** (or Lemon product URL) | Create product → copy share URL → same paste pattern | MoR-ish for digital goods; dashboard is the product. No Paddle.js. Integrations already Open Gumroad/Lemon. |
| **3** | **PayPal** button / link | Account you already know → one button URL | Lowest new learning if PayPal is already set up. You own VAT/invoices; refunds in PayPal. Not a MoR. |
| **4** | **Stripe Payment Link** | One link, still Stripe KYC | Simpler than Stripe Elements; still a payments platform, not “paste and forget.” |
| **Avoid** | Paddle overlay, Polar **API** E2E, Creem/Waffo as Solo, second Billing SDK | High | Same class of hassle we just rejected. |

**Recommended if money later:** **Polar checkout URL** (rank 1) — restore a link Buy on `/pricing`, not another overlay SDK. Polar was deferred only because Paddle was chosen as Solo processor; that choice is **withdrawn**.

**Recommended until motivation returns:** **Rank 0.** Do not start Polar account work in the same week as a Paddle halt.

## What we will not do

- Invest in Paddle Get Started, Paddle MCP prompt packs, or live KYC/domain for Solo.
- Dual-CTA (Paddle + Polar) on `/pricing`.
- Expand Desktop Integrations / coach to “replace” vendor dashboards.
- Auto-create vendor products from `orbityard`.

## Implementation (only if rank 1+ is activated)

1. Design slice: `/pricing` Buy = **external URL** (`PUBLIC_POLAR_CHECKOUT_URL` or generic `PUBLIC_CHECKOUT_URL`), CTA disabled when unset (same honesty as today).
2. Subtract Paddle.js from the Solo path (keep Integrations Paddle guide).
3. Fulfillment: keep manual `issue-license.py` / email until a webhook is worth it.
4. Proof: L3 = click Buy → vendor checkout page (not overlay).

Until then: **no code**. Handoff + CURRENT record the halt.
