# Frontend — Payments catalog (Creem / Waffo) + site landing honesty

**Product:** Ship Studio Desktop Integrations + official website  
**Audience:** Operators picking a checkout lane for *their* product; strangers on the public site  
**Reference:** Existing Desktop provider catalog · Linear marketing restraint on website  
**Stack:** Desktop vanilla TS (no React); website Astro + existing `global.css`  
**Spec status:** approved for this slice  
**API:** `shipctl` detect/portal/publish contracts in [commerce-creem-waffo-design](../../specs/backend/commerce-creem-waffo-design.md)

## Visual direction

- Desktop: reuse Payments card grid + wizard aside (no new chrome). Bundled SVG icons only.
- Website: same charcoal + emerald; fill the home void after hero with a **lane grid** (not a second pricing table).
- **NOT:** extra checkout buttons for Creem/Waffo/Paddle on `/pricing`. Never-say: Studio does not replace those vendors.

## Desktop — Integrations Payments

Archetype: catalog + wizard aside (existing).

Add cards after Paddle, before Email/Resend:

| id | title | blurb | openUrl |
|----|-------|-------|---------|
| creem | Creem | MoR checkout — keys stay on the host, not Studio. | https://creem.io/dashboard |
| waffo | Waffo | Waffo Pancake merchant dashboard — Studio never creates products. | https://pancake.waffo.ai/merchant/auth/signin |

Every Payments/Email wizard exposes `openLinks` (exact dashboard pages). Same Deploy-feel shape: Open pages → vendor work → Put when host secret · Confirm. Resend Put stays primary when a Tier A host is detected.

Paddle wizard (portal minute — sandbox first):

1. Primary **Open Paddle sandbox** plus page buttons (not docs): Catalog `/products` · Authentication `/authentication` · Checkout settings `/checkout-settings` · Orders `/orders` · sandbox signup. **Open is not Advanced-gated** — General + Public can land on the vendor page. Confirm listing still needs Advanced.
2. Do-X: copy `pri_…` and `test_…` · default payment link · Put `PUBLIC_PADDLE_*` outside Studio.
3. Sandbox is a **separate account**.
4. Continue publishing → Confirm `listing.paddle`.

Done lines and `INTEGRATION_PUBLISH_STEP` map to `listing.creem` / `listing.waffo`. Portal ids `creem` and `waffo` in `PORTAL_PROVIDER_IDS`. **Learn more** (`#int-docs`) uses catalog `docsUrl` (official tutorial — never the setup path).

Icons: `creem.svg` · `waffo.svg` in `apps/desktop/src/public/icons` (simple ink marks, not remote CDN).

## Website — `/` after hero

Purpose: one viewport already explains the hub; a second band explains **surfaces** and **commerce honesty**.

Layout:

1. Existing hero (unchanged CTA: Polar pricing or Download).
2. **Three surfaces** — CLI · TUI · Desktop (short lines).
3. **Payments honesty** — two columns:
   - *Your product:* Polar · Stripe · Gumroad · Lemon · Paddle · Creem · Waffo — Open vendor, Confirm in Studio.
   - *This site:* Solo Buy is Polar-hosted when `PUBLIC_POLAR_CHECKOUT_URL` is set; otherwise disabled with setup note.

DO NOT: list Creem/Waffo as ways to buy Ship Studio. DO NOT claim live Polar checkout.

## `/faq`

One row: “Can Studio take Paddle / Creem / Waffo payments for my app?” — Yes as a **guide** (Integrations). Studio Solo remains Polar.

## Wiring

| UI | Contract |
|----|----------|
| Desktop wizard Open | catalog `openUrl` |
| Continue publishing | `listing.creem` / `listing.waffo` / `listing.paddle` |
| Site Buy | `commerce.ts` Polar only |
