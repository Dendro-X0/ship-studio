# Polar checkout dogfood (W2)

End-to-end money path without putting card data on Ship Studio servers.

## 1. Create the product

1. Open [Polar dashboard](https://polar.sh/dashboard) (sandbox org if available).  
2. Create product **Ship Studio Solo** — one-time purchase.  
3. Copy the **Checkout** link (or Checkout API URL).  
4. Optional for Ship Studio Portal deep links: set `POLAR_ORGANIZATION_SLUG=<your-org-slug>` in the project `.env` so Open goes to Products / Settings / Webhooks instead of Overview.  

## 2. Wire redirect URLs on the Checkout Link

In Polar → **Products** → **Checkout Links** → your link → fill and **Save Link**:

| Polar field | Value (local dogfood) | Value (deployed site) |
|-------------|----------------------|------------------------|
| **Success URL** | `http://localhost:4321/checkout/success?checkout_id={CHECKOUT_ID}` | `https://<your-host>/checkout/success?checkout_id={CHECKOUT_ID}` |
| **Return URL** | `http://localhost:4321/checkout/cancel` | `https://<your-host>/checkout/cancel` |

Keep the literal `{CHECKOUT_ID}` tag in Success URL — Polar replaces it after payment.

If Polar rejects localhost, leave redirects blank for now, buy with a test card, then set Success/Return after the site is deployed.

## 3. Env on the website

```bash
cp apps/website/.env.example apps/website/.env
```

Set at least:

```env
PUBLIC_POLAR_CHECKOUT_URL=https://buy.polar.sh/...
PUBLIC_POLAR_PORTAL_URL=https://polar.sh/...
PUBLIC_POLAR_PRICE_LABEL=$49
PUBLIC_REFUND_WINDOW_DAYS=14
PUBLIC_SUPPORT_EMAIL=you@example.com
```

## 4. Prove

```bash
pnpm website:dev
# /pricing → Checkout with Polar (live link)
# Complete sandbox purchase → lands on /checkout/success
# Cancel mid-checkout → /checkout/cancel
# /legal/refunds → portal + support email
# /account → Polar customer portal
```

## 5. Refund path

1. Buyer opens Polar customer portal (`PUBLIC_POLAR_PORTAL_URL` / `/account`).  
2. Within `PUBLIC_REFUND_WINDOW_DAYS`, request refund in portal **or** email `PUBLIC_SUPPORT_EMAIL` with order ID.  
3. Maintainer approves in Polar — Studio never stores cards or issues refunds itself.

W4 — license keys + refund dogfood: [LICENSE-REFUND-DOGFOOD.md](./LICENSE-REFUND-DOGFOOD.md).
W2 stops at checkout + policy + redirects; W4 adds key-file issuance and revoke ledger.
