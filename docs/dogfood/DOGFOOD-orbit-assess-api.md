# Dogfood — Orbit / assess-api (2026-09-12, updated 2026-09-13)

Target: `E:/Web Projects/assess-api`

## Results

| Step | Result |
|------|--------|
| `shipctl doctor` | OK — nested wrangler + `.orbit` detected |
| `shipctl deploy -- status` | OK |
| Cloudflare API deploy | OK — `https://assess-api.paf437sywst688.workers.dev` |
| Worker secrets | **Blocked** — local `GITHUB_TOKEN` / Polar empty; `gh` not logged in. See assess-api `docs/SECRETS-OPS.md` |
| Health check from this host | **Blocked** — TCP timeout to `*.workers.dev` / Vercel (egress) |
| Docs (Vercel) deploy | **OK** after fixing npm lock (was pnpm-path polluted) + `vite` in dependencies |

## Docs URLs

- Production alias: https://docs-two-topaz.vercel.app
- Deployment: https://docs-ekj04vdk3-fictionalforges-projects.vercel.app

## Docs fix

`apps/docs/package-lock.json` previously pointed at monorepo `../../node_modules/.pnpm/...`. Regenerated a self-contained npm lock; `vercel.json` uses `npm ci`.

## Secrets (still need you)

```bash
cd "E:/Web Projects/assess-api/apps/api"
pnpm exec wrangler secret put GITHUB_TOKEN
pnpm exec wrangler secret put POLAR_WEBHOOK_SECRET
pnpm exec wrangler secret put POLAR_CHECKOUT_URL
```
