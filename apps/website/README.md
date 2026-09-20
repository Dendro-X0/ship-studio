# Ship Studio — official website

Public product surface (marketing · pricing · docs · demo · legal).  
Not part of the local Adaptive Publish hub — see `specs/backend/product-website-charter.md`.

## Develop

From repo root:

```bash
pnpm install
pnpm --filter ship-studio-website dev
# → http://localhost:4321
```

Or: `pnpm website:dev`

## Env (Polar placeholders)

Copy `.env.example` → `.env`:

| Variable | Purpose |
|----------|---------|
| `PUBLIC_POLAR_CHECKOUT_URL` | Solo checkout link (Polar dashboard) |
| `PUBLIC_POLAR_PORTAL_URL` | Customer portal (refunds / invoices) |
| `PUBLIC_DOWNLOAD_URL` | GitHub releases / installer |

Until set, Buy buttons show a “Checkout not configured” state.

## Build

```bash
pnpm --filter ship-studio-website build
```

Static output: `apps/website/dist` — deploy to Vercel / Cloudflare Pages / any static host.
