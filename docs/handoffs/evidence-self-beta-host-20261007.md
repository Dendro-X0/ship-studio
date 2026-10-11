# Evidence — self-beta host path — 2026-10-07

**Status:** Operator attestation + site smoke  
**Band:** OSS / personal ops · **not** commercial reopen  
**Upstream research:** `new-business/research/fixtures/private/self-intake/ship-orbit-self-beta-20261007.md`

## Claim

Internal beta complete for:

- Self-host  
- Self-sign  
- Guided / one-click-class deploy to **Vercel · Cloudflare · Netlify**  
- Dedicated website + GIF demos (`pnpm website:dev` → localhost:4321)

## Aligns with

| Doc | Note |
|-----|------|
| [PLATFORMS-AND-PORTAL](../product/PLATFORMS-AND-PORTAL.md) | Tier A Orbit deploy |
| [hosted-deploy-ops-value-bar](../../specs/backend/hosted-deploy-ops-value-bar-design.md) | H1–H5 shipped historically |
| CycleDesk ship-relaunch XOR | First public host URL |

## Still true (do not erase)

| Band | Rule |
|------|------|
| Commercial / DevOps GTM | CANCELLED until explicit reopen + ADMISSION |
| Integrations product | CANCELLED |
| Experience / service | Operator: **needs further polish** |

## Site smoke (local)

- `/` hero + honesty copy OK  
- `/demo` GIF stages present  
- Download fallback in repo: **v0.2.4** (`apps/website/src/lib/commerce.ts`) — confirm Netlify `PUBLIC_DOWNLOAD_URL` if production still pins an older tag

## Cloudflare Pages (Desktop Deploy · 2026-10-07 evening)

- Production: [https://ship-studio.pages.dev/](https://ship-studio.pages.dev/) — Orbit Yard marketing site live  
- Preview attested in Studio toast: `https://ccae2359.ship-studio.pages.dev`  
- Env import → Deploy path: toast **Env pushed → Deploy** · **Open live** with URL (host XOR S3)  
- Dashboard: Workers & Pages → **ship-studio** · No Git connection (CLI/wrangler upload)

## Next atomic (suggested)

[host-xor-value-ux-polish-design](../../specs/backend/host-xor-value-ux-polish-design.md) — **S1 Outcome contract** next; S3 live-URL toast dogfooded; S4 in-repo done.
