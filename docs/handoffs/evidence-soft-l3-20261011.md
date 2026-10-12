# Evidence — Soft-L3 self-timed Get-live (2026-10-11)

**Spec:** [presentation-readiness](../../specs/backend/presentation-readiness-design.md) R3 · [maintainer Soft-L3](./maintainer-soft-l3-get-live.md) · [stranger 20min](./stranger-20min-get-live-script.md)  
**Result:** **Pass** — live URL before 20:00 (incl. Login CLI gate + resume)  
**Surface:** CLI · `apps/website` · provider `cloudflare`

## Clock

| Mark | Wall |
|------|------|
| Start | 2026-10-11T03:43:06-12:00 |
| Auth probe → named gate | ~26s |
| Human: `wrangler login` | pause |
| Resume auth ok | 2026-10-11T03:48:18-12:00 |
| Deploy finished | 2026-10-11T15:49:08Z (UTC) · ~26s deploy |
| Open live smoke | HTTP 200 on preview + `ship-studio.pages.dev` |

Wall clock with pause for Login CLI still well under 20:00 of active Get-live work.

## Steps

1. Open project — `apps/website`  
2. Choose host — Cloudflare Pages · `--name ship-studio`  
3. Login once — first probe timed out → **Login CLI** (`wrangler login`); resume → `auth_required: false`  
4. Env + Deploy — `orbityard hostdeploy --project apps/website --provider cloudflare --name ship-studio`  
5. Proof — Open live · `apps/website/.ship/last-run.json` `ok: true`

## Live URL

- Preview (truth sync rebuild): [https://38eb02f3.ship-studio.pages.dev](https://38eb02f3.ship-studio.pages.dev)  
- Earlier preview (pre-rebuild): `https://712cd800.ship-studio.pages.dev`  
- Production alias: [https://ship-studio.pages.dev/](https://ship-studio.pages.dev/) — Download CTA → **v0.2.5**

`last-run` message: `hostdeploy cloudflare pages ok · https://38eb02f3.ship-studio.pages.dev`

## Download truth

First Soft-L3 deploy still served **v0.2.4** because `apps/website/.env` pinned `PUBLIC_DOWNLOAD_URL`. Rebuilt with override → redeploy → prod shows **v0.2.5**. Local `.env` updated to match.

## Named gate (resolved)

```text
cloudflare auth probe timed out — run Login CLI (`wrangler login`) [auth]
```

One recovery · no Integrations path · human finished Login CLI · resume completed deploy.

## Not claimed

- Desktop R4 GIF re-record (`/demo` shelf)  
- Netlify `PUBLIC_DOWNLOAD_URL` production env Put (repo fallback already v0.2.5)
