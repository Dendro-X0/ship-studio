# Owned commerce (Medusa) — study & decision

**Status:** Study complete — **Go** · **M1–M4 shipped** ([medusa-get-live](./medusa-get-live-design.md))  
**Updated:** 2026-10-10  
**Owner:** product / Publish General (future layout)  
**Parents:** [founder-minimal-spine](./founder-minimal-spine-design.md) · [ops-deploy-pivot](./ops-deploy-pivot-design.md) · [integrations-setup-is-not-the-product](./integrations-setup-is-not-the-product-design.md) · [commercial-product-failed](./commercial-product-failed-design.md) · research [146 digital value perception](E:/Web%20Projects/new-business/research/146-digital-value-perception-bias.md) · [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Medusa M4 done — see medusa-get-live-design.md
PAUSED / CANCELLED:  Integrations product · DevOps GTM · vendor coach ·
                     Portal/Integrations expansion · Solo commercial ADMIT
CANONICAL OWNER:     commerce.medusa under Get-live spine (not Integrations)
PROOF THIS BAND:     Desktop card + gate; Soft-L3 Admin = real Medusa clone
```

## 1. Verdict

| Decision | Sentence |
|----------|----------|
| **Go (parked until activated)** | Viable as a **host-XOR layout**: sequence user-owned Medusa onto their host — **not** as Integrations / Stripe setup. |

**Kill if** the band becomes “finish Stripe Connect inside Studio” or “sell Medusa zip as Solo spine.”  
**Park** code until handoff activates a thin detect+Deploy slice; study alone does not authorize diffs.

### Why this is not the Stripe/Paddle failure

| Failed wedge | Medusa-class wedge |
|--------------|-------------------|
| Finish **vendor** KYC / catalog / OAuth Studio cannot own | Sequence **user-owned** Medusa (OSS) on **their** host |
| Second encyclopedia of Polar/Paddle dashboards | Deploy + env Put + one loud gate; storefront stays user’s FE |
| “Integrations product” (CANCELLED) | Extension of founder spine: done = **observable admin/store URL** |

Integrations failure was a **layer** problem (walled gardens). Medusa is a **runtime the operator owns** — same class as “deploy this Node app,” with commerce domain knowledge only for detect + env names + proof URLs.

## 2. Job (founder outcome wall)

**Locked offer sentence:**

> Get a **self-hosted Medusa** backend live on the founder’s host — visible process, visible admin/store URL, honest Login/Put gates. Frontend and catalog customization stay with the operator. Orbit does not replace Stripe Connect / Paddle dashboards inside Medusa’s payment plugins.

Mapped to the five-step Get-live spine ([founder-minimal-spine](./founder-minimal-spine-design.md)):

| # | Spine step | Medusa meaning |
|---|------------|----------------|
| 1 | Open project | Bind folder with Medusa signals |
| 2 | Choose host | Prefer host that can run Node + Postgres (or Medusa Cloud later — out of band) |
| 3 | Login once | Host Login CLI / DB provision human gates |
| 4 | Env + Deploy | Put `DATABASE_URL` / Redis / JWT **names**; deploy API (+ optional admin static) |
| 5 | Open live · Confirm | Admin URL and/or storefront URL — human Confirm |

**Done signal:** HTTPS admin and/or storefront URL opens (or a **named** blocking gate with one recovery).  
**Not done:** “Stripe onboarding finished in Studio.”

## 3. Detect signals (proposed)

Any strong signal → layout `commerce.medusa` (name TBD in implement slice):

| Signal | Notes |
|--------|-------|
| `medusa-config.js` / `medusa-config.ts` | Canonical |
| `package.json` dependency `@medusajs/medusa` or `@medusajs/framework` | Monorepo-safe |
| `src/api` + Medusa scripts (`medusa develop` / `medusa start`) | Corroborate |
| Admin app markers (`@medusajs/admin` / `medusa-admin`) | Optional second pack |
| Next storefront (`@medusajs/js-sdk` + storefront README) | Separate ship unit — soft-no as required for Get-live API |

**Do not** detect “has Stripe” alone as Medusa.

## 4. Orbit owns vs user owns

| Orbit owns (sequencer) | User owns |
|------------------------|-----------|
| Detect layout + Adaptive Publish short path | Storefront theme / Next app UX |
| Doctor: Node, package manager, optional Docker/Postgres CLI | Catalog, products, customers in Medusa Admin |
| Hostdeploy / selfhost affinity for API (and docs for DB) | Payment plugin vendor UIs (Stripe Connect, etc.) |
| Env **name** Put / Import (never values in `.ship/`) | KYC, payouts, tax, domain email |
| MCP `yard_setup` spine map + pause/resume | Marketing site / brand |
| Soft-no: Integrations is not the Get-live path | Choosing Medusa vs Shopify vs nothing |

## 5. Soft-no / never-say

| Never-say | Say |
|-----------|-----|
| Orbit completes Stripe/Paddle inside Medusa | Payment plugins open the **vendor** dashboard; Confirm after you finish there |
| Buy our e-com zip / template as the product | Gift OSS Medusa + Orbit Get-live; paid only if H1 customize ADMIT |
| Integrations Payments is how you sell | Get live on Deployment / Publish |
| We replaced Medusa Admin | We sequence deploy; Admin stays Medusa |

## 6. Research 146 checklist (offer honesty)

| Spec | Medusa band |
|------|-------------|
| **V-Out** | Live Medusa admin (and/or storefront) URL on *their* host |
| **V-Proof** | Soft-L3: Open live after deploy; stranger script later |
| **V-Labor** | Orbit scripts deploy/env; human Login/DB/payment plugins |
| **V-Pain** | Avoid agency “set up Shopify/Stripe for me” scatter without owning the runtime |
| **V-Ritual** | Keys on their infra · admin URL · `.ship` progress |
| **V-Ethics** | No shame-piracy · no fake “we finished Connect” |
| **V-Segment** | Founders who already accept self-host / VPS bills (payable nearby) |

**Commercial posture:** Gift sequencer (OSS). Residual = optional **H1 install/customize** service track only after explicit ADMIT — not a mid SKU “download Medusa kit” ([146](E:/Web%20Projects/new-business/research/146-digital-value-perception-bias.md) §5–6). Does **not** reopen commercial-product-failed DevOps GTM.

## 7. Proof layers (when implement activates)

| Layer | Check |
|-------|--------|
| L1 | Fixture with `medusa-config` + detect reports Medusa layout |
| L2 | Local `medusa develop` (or docker compose) + doctor ok |
| L3 | Public admin or storefront URL after hostdeploy (timed Get-live) |
| L4 | Optional H1 on a third-party Medusa repo (service track) |

## 8. Risks

| Risk | Mitigation |
|------|------------|
| Scope creep into payment-plugin coach | Soft-no; Open vendor only |
| Medusa host needs Postgres — harder than static Pages | Honest gate: DB provision named; one recovery |
| Dual storefront + API = two deploys | Treat storefront as later/optional pack |
| Looks like Integrations revival in marketing | Copy: “self-hosted commerce runtime,” never “Stripe setup” |

## 9. Next atomic after this study

| If | Then |
|----|------|
| Maintainer activates | **M1–M4** shipped 2026-10-10 |
| Maintainer parks | Soft-L3 / FreeLLMAPI only when activated |
| Maintainer kills | Mark Status Kill |

**Next:** Soft-L3 on a real Medusa clone (Admin URL), or park.

**Commercial twin (2026-10-09):** Outcome SKU pressure-test — [outcome-sku-commerce-admin-medusa-2026](../../docs/business/outcome-sku-commerce-admin-medusa-2026.md) — **Go thin, standup-first** (Admin URL wall; not kit-as-OSS).

## Success criteria (this band)

- [x] Maintainer one-liner: **Go (parked) — owned Medusa Get-live ≠ Stripe Integrations**  
- [x] Separation from CANCELLED Integrations documented  
- [x] 146 V-* checklist filled without selling zip as spine  
- [x] **M1** fixture + detect shipped (2026-10-10)  
