# Outcome SKU pressure-test — Commerce admin live (Medusa)

**Status:** Pressure-test complete — **Go thin (standup-first; pack second)** · **not** Stripe/Integrations · **not** storefront theme  
**Updated:** 2026-10-09  
**Parent:** [OUTCOME-SKU-PAIN-DEMO-2026](./OUTCOME-SKU-PAIN-DEMO-2026.md) · [owned-commerce-medusa-study](../../specs/backend/owned-commerce-medusa-study-design.md) · [PEER-BUSINESS-MODELS-2026](./PEER-BUSINESS-MODELS-2026.md)  
**Candidate:** #2 Self-hosted commerce admin live — ownership / fee pain

## 1. Offer sentence (buyer words)

> Get a **Medusa Admin you own** live on your infrastructure — products and orders on *your* Postgres — without Shopify fees or category lock-in. Payments stay in the vendor dashboard you choose later.

**Done signal:** HTTPS Medusa Admin opens; you can sign in; API health ok.  
**Not done:** Stripe Connect finished · Next storefront theme · full catalog migration.

## 2. P1–P5 score

| # | Result | Notes |
|---|--------|-------|
| **P1** Spend nearby | **Pass** | Shopify apps/fees, agency quotes, Medusa Cloud $29–$299, VPS — buyers already pay |
| **P2** Observable demo | **Pass** | Admin URL + login screen = Soft-L3 (stronger “owned commerce” signal than Umami) |
| **P3** AI/clone hard | **Pass (stronger than #1)** | Dev Docker ≠ production: Postgres + Redis, server **and** worker, secrets, migrations, ≥~2GB RAM, CORS, file storage. Free tutorials exist but **production wall is real** — not a 15‑minute Umami compose |
| **P4** Finite delivery | **Pass if scoped** | Admin+API live once. Kill if SKU includes storefront + payments + migration |
| **P5** Channel | **Fail until named** | Agencies already sell Medusa; cold GitHub won’t win. Need Shopify-pain / hitch / community — same block as #1 |

**Verdict:** **Go thin — prefer H1 standup over kit-first.** Hardened pack is viable *after* you can demo production Soft-L3. Aligns with owned-commerce study; does **not** reopen Integrations.

**vs Umami (#1):** Higher willingness-to-pay and stronger P3; heavier delivery; longer Soft-L3; Medusa Cloud is the convenient substitute (sell ownership / no GMV tax / leave Shopify).

## 3. What we sell vs what we don’t

| In (SKU) | Out |
|----------|-----|
| Production path: Postgres + Redis + server + worker + env secrets + migrate + HTTPS Admin | Finishing Stripe/Paddle/KYC in Studio |
| Runbook: DNS → up → create admin → Confirm Admin URL | Next.js storefront design / theme |
| Soft-L3 video: empty host → Admin login | Catalog import / SEO migration (later add-on) |
| **H1 standup** ~$399–$799 (half–day class; above template band — real ops) | Unlimited RUN retainer as day-one |
| Optional **kit** ~$199–$299 after Soft-L3 exists (compose/Railway recipe + secrets checklist) | “Medusa zip as Orbit Solo” · Gift dump |

**Orbit role:** Later detect+Deploy (M1) helps *delivery*; monetizable product is still **Admin live**, not the sequencer license.

## 4. Fake landing (copy only)

**Headline:** Your commerce backend. Your database. No Shopify tax.  
**Sub:** Production Medusa Admin live on your host — server, worker, Postgres, Redis — so you own the ops layer. Storefront and payments stay yours to choose.  
**Proof:** 2–3 min Soft-L3 — Admin URL + login (not a 90s Umami clip; allow longer).  
**Price:**  
- **Done-for-you** $499 — Admin live on your VPS/Railway once (you provide host + DNS)  
- **Kit** $249 — production recipe + runbook (after demo ships)  
**Soft-no:** We do not finish Stripe inside Medusa. We do not replace Medusa Cloud if you want zero ops. We do not build your storefront in this SKU.

## 5. Demo script (~150s)

| t | Show |
|---|------|
| 0:00–0:20 | Pain: Shopify fees / lock-in / “I want the backend” |
| 0:20–0:70 | Host: stack up (compose or Railway) — Postgres/Redis healthy |
| 0:70–1:40 | Migrate · start server+worker · HTTPS Admin loads |
| 1:40–2:20 | Login · empty products list = **owned** Admin |
| 2:20–2:30 | Soft-no one line: payments = Open vendor later · CTA standup/kit |

## 6. Risks

| Risk | Mitigation |
|------|------------|
| Looks like Integrations revival | Copy: Admin URL wall only; never “Stripe setup” |
| Medusa Cloud cheaper in effort | Segment: must self-host / leave Shopify / own DB |
| Scope → storefront + plugins | Hard scope; separate SKUs later |
| Agencies undercut on relationships | Don’t compete on brand Medusa partner — sell **finite Admin live** to founders who won’t hire an agency |
| P5 | No pack/public repo until channel or Soft-L3 for hitch |

## 7. Ranking vs candidate #1

| | Umami (#1) | Medusa (#2) |
|--|------------|-------------|
| P3 scarcity | Weak | **Stronger** |
| Soft-L3 cost | Low | Higher |
| Kit alone | Risky | Riskier until demo |
| H1 first | Optional | **Recommended** |
| Orbit synergy | Weak (VPS compose) | Stronger (future M1 layout) |

**Portfolio note:** If energy for only one Soft-L3 soon → **#1 faster to film**; if energy for one **higher-ticket** wall → **#2 H1**. Both still need P5.

## 8. Next atomic

| If | Then |
|----|------|
| Choose Medusa commercial focus | Soft-L3 Admin live on throwaway host · then H1 offer paper with $499 wall |
| Keep Umami as faster wedge | Finish #1 90s clip first; park Medusa pack |
| Kill | Only if SKU drifts to payments/storefront theater |

**Recommended default:** Do **not** build a Medusa zip yet. Either film Admin Soft-L3 **or** keep #1 as the quicker proof of the outcome-SKU grammar.
