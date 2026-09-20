# Official product website — charter

**Status:** W0 done (first slice) — scaffold + Polar placeholders + static pages  
**Updated:** 2026-09-19  
**Owner:** `apps/website` · commerce via Polar (preferred)  
**Parent:** product commercialization · separate from Adaptive Publish hub  

## W0 acceptance

- [x] `apps/website` Astro static site in pnpm workspace  
- [x] `/` hero (brand · headline · CTAs)  
- [x] `/pricing` with Polar checkout placeholder (`PUBLIC_POLAR_CHECKOUT_URL`)  
- [x] `/docs` index linking repo docs  
- [x] `/demo` placeholder stage  
- [x] `/legal/refunds` · terms · privacy stubs  
- [x] `pnpm website:build` succeeds  

## Next

**W1:** render `docs/product` Markdown into `/docs/*`  


## Boundary (do not conflate)

| Surface | Role |
|---------|------|
| **Ship Studio** (`shipctl` · TUI · Desktop) | Local shipping hub — stays offline-first; does **not** author marketing docs/demos |
| **Official website** (this charter) | Public product presence: pitch · buy · refund · docs · demo |

Studio may later **detect** `apps/website` as `marketing.deploy` for *customer* projects. This repo’s own marketing site is a **sibling product surface**, not an Adaptive Publish lane inside the bridge.

## Goal

Ship a public site that lets a stranger:

1. Understand what Ship Studio is in one viewport  
2. Buy a license / plan with honest checkout  
3. Request or receive a refund under a published policy  
4. Read documentation (install · Publish spine · human gates)  
5. Watch or play a **feature demo** (silent GIF / short walk) without installing first  

## Recommended stack (first slice)

| Piece | Choice | Why |
|-------|--------|-----|
| Site | Astro or Next static + light islands | Docs + marketing; deploy on Vercel/Cloudflare Pages |
| Path | `apps/website` | Matches Studio’s marketing.deploy detection later |
| Payments | **Polar** Checkout / Product links | Already in Studio commerce honesty; OSS-friendly; less PCI surface |
| Refunds | Polar refund API + public `/legal/refunds` policy | Human-triggered + automated window (e.g. 14 days) |
| Docs | Publish `docs/product` + START-HERE as MD → site `/docs` | One source of truth; no duplicate prose in Studio |
| Demo | Silent GIFs + 60–90s scripted walk of Desktop Publish | Obscur-style GIF shelf; no fake “cloud Studio” |

**Avoid first slice:** building a SaaS control plane, storing customer secrets, replacing Polar’s customer portal, or embedding live `shipctl` in the browser.

## Site map

```
/                 Hero: brand · one headline · one CTA (Download / Buy)
/pricing          Plans · Polar checkout buttons · what’s included
/docs             Index → install · Publish · Launch · OPERATOR-NEXT · FAQ
/docs/*           Generated from docs/ + selected specs (north star summary only)
/demo             Feature demo page (GIF + captions + “Open desktop” CTA)
/legal/terms
/legal/privacy
/legal/refunds    Clear window, how to request, what is non-refundable
/account          Optional later — Polar customer portal link (not custom billing UI)
```

## Payment + refund integration

### Payments

1. Create Polar products (e.g. Solo license · optional Pro later).  
2. Website CTAs → Polar Checkout URL / overlay (no card data on our servers).  
3. Success URL → `/docs/start` + download instructions.  
4. Webhook (optional first-mile): record purchase email → license key issuance (file or simple Cloudflare Worker + KV).  

### Refunds

| Policy (proposed) | Detail |
|-------------------|--------|
| Window | 14 days from purchase (configurable) |
| Path | Link to Polar customer portal **or** form → maintainer triggers Polar refund |
| Auto | Worker calls Polar refund API when request is in-window and unused license |
| Honesty | No “instant cloud cancel” fiction — Studio is a local tool; refund is money-back, not remote wipe |

Legal copy lives under `/legal/refunds`; Studio app never processes card refunds.

## Documentation section

| Source (repo) | Public page |
|---------------|-------------|
| `docs/START-HERE.md` | `/docs` intro |
| `docs/product/SCOPE-OF-SERVICE.md` | `/docs/scope` |
| `docs/product/PRODUCT.md` | `/docs/product` |
| `docs/product/OPERATOR-NEXT.md` | `/docs/human-gates` |
| `docs/dogfood/*` | `/docs/dogfood` (optional) |
| North star (summary only) | `/docs/roadmap` — link to GitHub for full bands |

CI: `pnpm docs:build` fails if a linked MD path breaks (lightweight check).

## Feature demo

1. **Silent GIFs** (20–30 FPS): bind project → Publish Open → Confirm → Next · Output Preview.  
2. **Script:** `docs/assets/demo/v0.1.0/` (presenter checklist + captions).  
3. **Page:** `/demo` — one composition, brand first, one CTA (Buy / Download).  
4. **Honesty:** watermark “local app · recorded” — no simulated cloud dashboard.

## Delivery bands (suggested)

| Band | Slice | Proof |
|------|-------|-------|
| **W0** | Charter approved · Polar product stub · `apps/website` scaffold | Site builds · `/` hero |
| **W1** | Docs pipeline from `docs/product` | `/docs` renders SCOPE + START-HERE |
| **W2** | Pricing + Polar checkout + refund policy page | Test purchase in Polar sandbox |
| **W3** | Feature demo page + first GIF set | `/demo` L3 walkthrough |
| **W4** | License delivery (email / key file) + refund path dogfood | End-to-end buy → refund |

PAUSED Adaptive bands (store API upload, k8s, aperio L4) stay parked — website work does not reopen them.

## Explicit non-goals (W0–W2)

- Multi-tenant cloud Ship Studio  
- In-app purchase inside the Tauri shell  
- Replacing Polar/Stripe customer billing UIs  
- Authoring demos *inside* `shipctl`  

## Next atomic step (when maintainer says go)

**W0:** scaffold `apps/website` + Polar product placeholders + deploy preview URL; update `docs/CURRENT.md` status to “product site W0”.
