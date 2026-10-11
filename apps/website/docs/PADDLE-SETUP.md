# Paddle — Solo checkout on this website

**What this is:** Optional Buy overlay on `/pricing` for **Orbit Yard Solo** (this site).  
**What this is not:** Desktop Integrations product · DevOps GTM · Studio finishing Paddle OAuth for *your* customers ([solo-commerce-without-paddle](../../../specs/backend/solo-commerce-without-paddle-design.md) · commercial ADMIT still cancelled).

Paddle has more gates than Polar. Collapse what you can with a seed script; finish the one dashboard setting Paddle requires; keep `pdl_…` out of Studio and git.

Canon: [website-paddle-checkout-design](../../../specs/backend/website-paddle-checkout-design.md) · [website-paddle-checkout-spec](../../../docs/frontend/website-paddle-checkout-spec.md).  
License after pay: [LICENSE-REFUND-DOGFOOD.md](./LICENSE-REFUND-DOGFOOD.md).

Stay on **Sandbox** until a test card completes. Live is real money. Sandbox and live have **separate** products, price ids, and tokens.

---

## Operator checklist (dashboard → `.env` → redeploy)

Do this when you are ready to turn on `/pricing` Checkout.

### A. Paddle dashboard (Sandbox first)

1. Open **[sandbox-vendors.paddle.com](https://sandbox-vendors.paddle.com/)** (not live paddle.com alone — sandbox is a separate signup).
2. **Catalog** → create one-time product **Orbit Yard Solo** ($29) → copy **Price ID** (`pri_…`).
3. **Authentication** → **Client-side tokens** → create token → copy (`test_…`).  
   Optional for seed script only: API key `pdl_sdbx_…` with `product.write` / `price.write` / `client_token.write` — **never** put `pdl_…` in `PUBLIC_*`.
4. **Checkout settings** → set **default payment link** (required to create transactions).

   Dashboard validation often **rejects** `http://localhost:4321` and even `https://localhost/`. Practical path:

   | Goal | Default payment link |
   |------|----------------------|
   | **Dogfood (preferred)** | `https://ship-studio.pages.dev/` (must Save when the green check appears) |
   | Local-only | Only if the field accepts a localhost variant; otherwise test Checkout on **production** after Put + Redeploy |

   Soft-no: do not burn time on localhost approval — Put `PUBLIC_PADDLE_*` on Pages and redeploy, then open `/pricing` on the live origin.

Without the default payment link, overlay checkout fails (“Something went wrong”).

### B. Local `.env` (gitignored)

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
PUBLIC_DOWNLOAD_URL=https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.5
```

Or seed from an API key (writes the same `PUBLIC_PADDLE_*` keys):

```bash
setx PADDLE_SANDBOX_API_KEY "pdl_sdbx_…"   # Windows; new terminal after
# export PADDLE_SANDBOX_API_KEY=pdl_sdbx_…  # Git Bash / macOS / Linux
python scripts/seed-paddle-solo-sandbox.py
```

### C. Rebuild + redeploy Cloudflare Pages (required for live `/pricing`)

This site is **direct-upload** Pages (`wrangler pages deploy dist`) — not a Git build that reads dashboard env. Astro inlines `PUBLIC_*` when **`astro build`** runs on the machine that produces `dist/`.

| Path | What to do |
|------|------------|
| **Correct** | Fill `apps/website/.env` → `pnpm website:build` → Desktop **Deploy** (or `hostdeploy`) |
| **Insufficient alone** | Import env / `pages secret put` — Secrets do **not** rewrite already-built static HTML for `PUBLIC_*` |

**Desktop:** ensure `.env` is filled → from repo root run build (or Studio build if available) → Deployment → Cloudflare **ship-studio** → **Deploy**.

**CLI:**

```bash
pnpm website:build
orbityard hostdeploy --project apps/website --provider cloudflare --name ship-studio
```

Optional: still Import env for future Git/SSR binding — it does not fix Checkout on the current static cut.

### D. Prove on production (or local if default link allows)

Open `https://ship-studio.pages.dev/pricing` → **Checkout with Paddle** → test card `4242 4242 4242 4242` · future expiry · CVC `100`.  
Success → `/checkout/success`. Orders: [sandbox Orders](https://sandbox-vendors.paddle.com/orders).

**Attested Pass (2026-10-07):** overlay → Pay → bank contact → “You’re in.” — [evidence](../../../docs/handoffs/evidence-paddle-sandbox-checkout-20261007.md).

Optional local: `pnpm website:dev` only if default payment link accepts a localhost URL (often it does not).

---

## Fast path summary

| Step | Where |
|------|--------|
| 1. Product + `pri_…` + `test_…` | Paddle Sandbox dashboard |
| 2. Default payment link | Checkout settings |
| 3. Paste into `apps/website/.env` | Local (gitignored) |
| 4. `pnpm website:dev` test card | Local proof |
| 5. Put env on host + Deploy | Cloudflare Pages / Netlify |

## Refunds

Buyers request via `/legal/refunds` (email + transaction id). You refund on [Orders](https://sandbox-vendors.paddle.com/orders).

- **Sandbox:** refund adjustments auto-approve about every 10 minutes.
- **Live:** Paddle usually must approve the refund.

```bash
python scripts/issue-license.py revoke --order paddle_xxx --note "paddle refund <id>"
python scripts/issue-license.py issue --email buyer@example.com --order paddle_xxx
```

Issue a license only after you see the paid transaction.

## What you will never put in Studio / git

| Keep in Paddle | Put on the website host |
|----------------|-------------------------|
| Catalog, refunds, invoices, customer emails | `PUBLIC_PADDLE_CLIENT_TOKEN`, `PUBLIC_PADDLE_PRICE_ID`, support email, refund window |
| API keys (`pdl_…`) | User env / seed script only — never `PUBLIC_*` · never commit `.env` |

## Soft no

- Desktop **Integrations → Paddle** is an optional bookmark, not required for this checklist.
- Turning on Buy does **not** reopen commercial ADMIT or Integrations-as-product.
- Do not claim live checkout works until host env + redeploy are done.

## Cursor MCP (optional)

Same sandbox API key can drive Paddle’s MCP for catalog. It does **not** replace overlay env or the default payment link.

```bash
# ~/.bashrc — then restart Cursor
export PADDLE_SANDBOX_API_KEY="pdl_sdbx_…"
```
