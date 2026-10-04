# Paddle — Solo checkout and refunds

Paddle has more gates than Polar or Stripe. Collapse what you can with a host seed; finish the one dashboard setting Paddle requires; keep `pdl_…` out of Studio and git.

Canon: [website-paddle-checkout-design](../../../specs/backend/website-paddle-checkout-design.md) · [payments-setup-path-optimization-design](../../../specs/backend/payments-setup-path-optimization-design.md).  
License after pay: [LICENSE-REFUND-DOGFOOD.md](./LICENSE-REFUND-DOGFOOD.md).

Stay on **Sandbox** until a test card completes. Live is real money. Sandbox and live have **separate** products, price ids, and tokens.

## Fast path (preferred)

### 1. Sandbox account

Sandbox is **not** a toggle on a live signup. If you only used paddle.com, open [sandbox signup](https://sandbox-vendors.paddle.com/signup). Then use Desktop Integrations → Paddle page buttons (skip Get started).

### 2. Seed catalog + client token

Create a sandbox **API key** (`pdl_sdbx_…`) at [Authentication](https://sandbox-vendors.paddle.com/authentication) with `product.write`, `price.write`, and `client_token.write`. User env only:

```bash
setx PADDLE_SANDBOX_API_KEY "pdl_sdbx_…"
# new terminal
python scripts/seed-paddle-solo-sandbox.py
```

Creates **Orbit Yard Solo** ($29 one-time) + a `test_…` token and writes `PUBLIC_PADDLE_*` into `apps/website/.env` (gitignored).

Do **not** put `pdl_…` in `PUBLIC_*`.

### 3. Default payment link (required)

Open [Checkout configuration](https://sandbox-vendors.paddle.com/checkout-settings) → **default payment link**.

Without this, overlay checkout fails (“Something went wrong”).

- Local: `http://localhost:4321`
- Live: approved public origin ([website approval](https://developer.paddle.com/paddle-js/about/))

### 4. Prove checkout

```bash
pnpm website:dev
```

`/pricing` → Checkout with Paddle → test card `4242 4242 4242 4242` · future expiry · CVC `100`.

Success → `/checkout/success`. Paid tests: [Orders](https://sandbox-vendors.paddle.com/orders).

Webhooks / Get started “fulfillment” are **not** required for this first sandbox buy. Solo license issue stays operator-side (`issue-license.py`) until you add webhooks later.

## Manual path (no seed)

[Catalog](https://sandbox-vendors.paddle.com/products) → one-time `pri_…` · [Authentication](https://sandbox-vendors.paddle.com/authentication) → Client-side tokens → `test_…` · same Checkout configuration step · paste into `.env`:

```bash
cp apps/website/.env.example apps/website/.env
```

```env
PUBLIC_PADDLE_CLIENT_TOKEN=test_…
PUBLIC_PADDLE_PRICE_ID=pri_…
PUBLIC_PADDLE_ENV=sandbox
PUBLIC_PADDLE_PRICE_LABEL=$29
PUBLIC_PADDLE_PORTAL_URL=
PUBLIC_REFUND_WINDOW_DAYS=14
PUBLIC_SUPPORT_EMAIL=you@example.com
PUBLIC_DOWNLOAD_URL=https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.3
```

`PUBLIC_PADDLE_PORTAL_URL` is optional (Paddle **customer** portal only). Leave blank so `/account` does not send buyers to the vendor dashboard.

## Refunds

Buyers request via `/legal/refunds` (email + transaction id). You refund on [Orders](https://sandbox-vendors.paddle.com/orders).

- **Sandbox:** refund adjustments auto-approve about every 10 minutes.
- **Live:** Paddle usually must approve the refund.

```bash
python scripts/issue-license.py revoke --order paddle_xxx --note "paddle refund <id>"
python scripts/issue-license.py issue --email buyer@example.com --order paddle_xxx
```

Issue a license only after you see the paid transaction.

## What you will never put in Studio

| Keep in Paddle | Put on the website host |
|----------------|-------------------------|
| Catalog, refunds, invoices, customer emails | `PUBLIC_PADDLE_CLIENT_TOKEN`, `PUBLIC_PADDLE_PRICE_ID`, support email, refund window |
| API keys (`pdl_…`) | User env / Cursor MCP `PADDLE_SANDBOX_API_KEY` — never `PUBLIC_*` |

## Cursor MCP (optional)

Same sandbox API key can drive Paddle’s MCP for catalog. It does **not** replace overlay env or the default payment link.

1. `setx PADDLE_SANDBOX_API_KEY "pdl_sdbx_…"` · restart Cursor.  
2. MCP: `paddle-sandbox` · `paddle-docs`. Approve each `execute`.  
3. Still need `PUBLIC_PADDLE_*` in `apps/website/.env` (seed or paste).

Desktop Integrations Paddle is a **guide**. Solo sale is this overlay + this env file.
