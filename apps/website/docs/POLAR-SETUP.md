# Polar checkout dogfood (W2)

End-to-end money path without putting card data on Ship Studio servers.

## 1. Create the product

1. Open [Polar dashboard](https://polar.sh/dashboard) (sandbox org if available).  
2. Create product **Ship Studio Solo** — one-time purchase.  
3. Copy the **Checkout** link (or Checkout API URL).  

## 2. Wire redirect URLs in Polar

| Polar setting | Value |
|---------------|--------|
| Success URL | `https://<host>/checkout/success` |
| Cancel URL | `https://<host>/checkout/cancel` |

Local dogfood: use a tunnel (e.g. Cloudflare Tunnel) or Polar’s allowed localhost if supported; otherwise test redirects after first static deploy.

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
