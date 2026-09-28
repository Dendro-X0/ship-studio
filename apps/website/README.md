# Ship Studio — official website

Public product surface (marketing · pricing · docs · demo · legal · checkout).  
Not part of the local Adaptive Publish hub — see `specs/backend/product-website-charter.md`.

## Develop

From repo root:

```bash
pnpm install
pnpm website:dev
# → http://localhost:4321
```

## Polar checkout (W2)

Full dogfood steps: [docs/POLAR-SETUP.md](./docs/POLAR-SETUP.md).

```bash
cp apps/website/.env.example apps/website/.env
# set PUBLIC_POLAR_CHECKOUT_URL (+ portal, price label, support email)
```

| Variable | Purpose |
|----------|---------|
| `PUBLIC_POLAR_CHECKOUT_URL` | Solo checkout link |
| `PUBLIC_POLAR_PORTAL_URL` | Customer portal |
| `PUBLIC_POLAR_PRICE_LABEL` | Display price (e.g. `$49`) |
| `PUBLIC_REFUND_WINDOW_DAYS` | Default `14` |
| `PUBLIC_SUPPORT_EMAIL` | Refund / support mailto |
| `PUBLIC_DOWNLOAD_URL` | GitHub releases |

Polar success/cancel URLs should hit `/checkout/success` and `/checkout/cancel`.

Until `PUBLIC_POLAR_CHECKOUT_URL` is set, Buy stays disabled with a setup note.

## Never-say (S1.3)

Public copy must not claim: one-click deploy · silencing SmartScreen · replacing Cloudflare/Polar/Apple · Studio-held deploy secrets · auto-publish to stores.  
Canon: [never-say-block-design](../../specs/frontend/never-say-block-design.md).

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
python scripts/issue-license.py issue --email buyer@example.com --order polar_xxx
python scripts/issue-license.py revoke --order polar_xxx
```

Format: `docs/assets/license/FORMAT.md`. Ledger: `.ship-licenses/` (gitignored).

## Build

```bash
pnpm website:build
```

Static output: `apps/website/dist`.
