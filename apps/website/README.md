# Orbit Yard — official website

Public surface (docs · demo · honesty · free Download). **Not for sale** — Solo / Paddle checkout cancelled.  
Not part of the local Adaptive Publish hub — see `specs/backend/product-website-charter.md`.

## Develop

From repo root:

```bash
pnpm install
pnpm website:dev
# → http://localhost:4321
```

## Distribution

| Variable | Purpose |
|----------|---------|
| `PUBLIC_DOWNLOAD_URL` | GitHub Releases tag (default matches CURRENT) |
| `PUBLIC_SUPPORT_EMAIL` | Optional maintainer mailto |

`/pricing` is **Get it free** ($0). Historical Paddle notes: [docs/PADDLE-SETUP.md](./docs/PADDLE-SETUP.md) (**CANCELLED** — do not re-enable sale UI).

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

## Build · deploy

```bash
pnpm website:build
# Static output: apps/website/dist
```

**Self-beta live host:** Cloudflare Pages project **ship-studio** (`https://ship-studio.pages.dev`).  
**Alternate:** Netlify (`netlify.toml` · base directory `apps/website`).

```bash
orbityard hostdeploy --project apps/website --provider cloudflare --name ship-studio
# or Netlify: orbityard hostdeploy --project apps/website --provider netlify
```

Put `PUBLIC_PADDLE_*` (and support/download) on the **same** host env before/with Deploy — Astro inlines `PUBLIC_*` at build time. Never commit `.env`.
