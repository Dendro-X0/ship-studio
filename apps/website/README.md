# Orbit Yard — official website

Public product surface (marketing · pricing · docs · demo · legal · checkout).  
Not part of the local Adaptive Publish hub — see `specs/backend/product-website-charter.md`.

## Develop

From repo root:

```bash
pnpm install
pnpm website:dev
# → http://localhost:4321
```

## Paddle checkout (Studio Solo)

Full dogfood steps: [docs/PADDLE-SETUP.md](./docs/PADDLE-SETUP.md).

```bash
cp apps/website/.env.example apps/website/.env
# set PUBLIC_PADDLE_CLIENT_TOKEN + PUBLIC_PADDLE_PRICE_ID (sandbox first)
```

| Variable | Purpose |
|----------|---------|
| `PUBLIC_PADDLE_CLIENT_TOKEN` | Client-side token (`test_…` / `live_…`) |
| `PUBLIC_PADDLE_PRICE_ID` | Catalog price id (`pri_…`) |
| `PUBLIC_PADDLE_ENV` | `sandbox` (default) or `live` |
| `PUBLIC_PADDLE_PRICE_LABEL` | Display price (e.g. `$29`) |
| `PUBLIC_PADDLE_PORTAL_URL` | Optional customer portal URL (leave blank) |
| `PUBLIC_REFUND_WINDOW_DAYS` | Default `14` |
| `PUBLIC_SUPPORT_EMAIL` | Refund / support mailto |
| `PUBLIC_DOWNLOAD_URL` | GitHub releases |

Until token and price id are set, Buy stays disabled with a setup note. Overlay success redirects to `/checkout/success`.

## Never-say (S1.3)

Public copy must not claim: one-click deploy · silencing SmartScreen · replacing Cloudflare/Polar/Apple · Studio-held deploy secrets · auto-publish to stores.  
Canon: [never-say-block-design](../../specs/frontend/never-say-block-design.md).

## Intent (S1.4)

`/intent` — multi-surface final-mile vs CI-only / checklist theater.  
Canon: [one-intent-page-design](../../specs/frontend/one-intent-page-design.md).

## Honesty (S1.5)

`/honesty` — human gates are the product (Open → Confirm), not unfinished bugs.  
Canon: [operator-next-product-design](../../specs/frontend/operator-next-product-design.md) · docs: `/docs/human-gates`.

## FAQ (S1.6)

`/faq` — SmartScreen · vs ship.studio · why Solo · Signet required.  
Canon: [cafe-faq-design](../../specs/frontend/cafe-faq-design.md).

## Feature demo (W3)

`/demo` serves live Harbor GIFs from `public/demo/v0.2.1/` (**T1–T7**).  
Script + captions: `docs/assets/demo/v0.2.1/SCRIPT.md`.

Legacy stylized set remains under `public/demo/v0.1.0/`:

```bash
python scripts/generate-demo-gifs.py   # regenerate stylized GIFs → docs + public (v0.1.0 only)
```

## License + refund dogfood (W4)

Buyer page: `/license` · maintainer checklist: [docs/LICENSE-REFUND-DOGFOOD.md](./docs/LICENSE-REFUND-DOGFOOD.md).

```bash
python scripts/issue-license.py issue --email buyer@example.com --order paddle_xxx
python scripts/issue-license.py revoke --order paddle_xxx
```

Format: `docs/assets/license/FORMAT.md`. Ledger: `.ship-licenses/` (gitignored).

## Build · deploy (Netlify)

```bash
pnpm website:build
# Static output: apps/website/dist
```

**Host:** Netlify (`netlify.toml`). Link the monorepo with Base directory `apps/website`, or from that folder:

```bash
netlify login
orbityard hostdeploy --project apps/website --provider netlify
# or: orbityard publish → Deploy (after Login CLI)
```

Put `PUBLIC_PADDLE_*` on the Netlify site env (not in git) for Solo Buy.
