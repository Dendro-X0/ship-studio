# Frontend — Solo Buy via Paddle overlay

**Product:** Official website `/pricing`  
**Spec status:** implemented — overlay gated on env; L3 overlay unproven until tokens exist  
**API:** [website-paddle-checkout-design](../../specs/backend/website-paddle-checkout-design.md)

## Page: Pricing

Archetype: marketing price block (existing).

Primary CTA:

- **Configured:** button `Checkout with Paddle` — `type="button"`, opens overlay (not a Polar URL).
- **Unconfigured:** disabled button + note to set `PUBLIC_PADDLE_CLIENT_TOKEN` and `PUBLIC_PADDLE_PRICE_ID`.

Load `https://cdn.paddle.com/paddle/v2/paddle.js` only on `/pricing` when configured.

Success: Paddle `settings.successUrl` → `/checkout/success`. Cancel: overlay close stays on `/pricing` (no Polar return URL required).

## Copy

Solo processor name is **Paddle**. Polar is not mentioned as the way to buy this license. Integrations guide on `/` may still list Polar for *customer* products.

`/account` and `/legal/refunds` are buyer-facing: receipt email + support mailto. Do not link buyers to `vendors.paddle.com`. Optional `PUBLIC_PADDLE_PORTAL_URL` only if a Paddle **customer** portal exists.

## DO NOT

- Inline checkout iframe redesign
- Dual Polar + Paddle Buy buttons
- Put `pdl_…` server API keys in `PUBLIC_*`
- Claim live checkout works without env
