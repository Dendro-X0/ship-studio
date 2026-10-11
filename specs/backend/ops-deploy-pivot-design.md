# Pivot — Ops and deploy; no third-party integration product

**Status:** Maintainer decision — **CANCELLED** Integrations-as-product  
**Updated:** 2026-10-04  
**Owner:** product contract  
**Parents:** [integrations-setup-is-not-the-product](./integrations-setup-is-not-the-product-design.md) · [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Idle → this charter replaces Integrations expansion
PAUSED / CANCELLED:  vendor coach; Paddle Solo; Integrations UX expansion → CANCELLED as a line
CANONICAL OWNER:     Publish / deploy / sign (not listing.polar)
PROOF THIS SLICE:    docs/contract only — no Desktop deletion yet
```

## Decision

Orbit Yard is an **ops and deployment tool** for **this repo’s** cut: detect → sign → release → **deploy**.

It **abandons** third-party **integration** as a product: Polar, Paddle, Stripe, Gumroad, Lemon, Creem, Waffo, Resend/email onboarding, “complete checkout in Studio,” commerce listing wizards as a competitive feature.

## Why (maintainer)

Vendors built **walled gardens** and technical barriers. There is no unofficial streamlined setup path. Users either endure official docs or pick a provider that is already easy. Competing there **reduces** Orbit’s value and is **not implementable** without owning a payments stack or a cloud.

If Orbit owned billing or hosting, “seamless setup” would be **Orbit’s own derivative service**, not a Polar/Paddle integration. We do not own those systems. We will not pretend.

This makes the product **narrower and more viable**. It does not make Polar easier. Polar stays Polar.

## Keep vs cancel

| Keep (ops / deploy) | Cancel (integration product) |
|---------------------|------------------------------|
| Detect layout (Wrangler, Netlify, Tauri, CI) | Payments / Email **Integrations** as a setup product |
| Adaptive Publish: sign → GitHub/Signet release → **host deploy** | Polar / Paddle / Stripe / Gumroad / Lemon / Creem / Waffo **onboarding** |
| Login CLI + Put **deploy** env names (host already chosen) | Solo Buy overlay / checkout URL as an Orbit feature |
| Doctor, Pulse, one-gate pause/resume for **current deploy/sign gate** | Vendor MCP prompt packs, coach, encyclopedia wizards |
| Open host dashboard to **confirm a deploy you ran** | “Help you set up Polar without the Polar UI” |
| Marketplace **submit** later only as ops (you already have Steamworks) — not store signup | Growing listing.* commerce lanes |

**Host deploy is not “Polar integration.”** `netlify deploy` / `wrangler` after **you** authenticated is ops. Creating a Polar org, product, and checkout is not.

## Subtract later (do not do in this slice)

When activated: hide or delete Desktop Integrations Payments/Email as a primary surface; drop commerce from Launch opt-in copy; stop treating `listing.polar` as a finishability goal; website `/pricing` stays Download-first.

Until then: **do not expand** those UIs. Dead code can wait one subtract slice.

## Never-say

- Orbit completes third-party integrations  
- Orbit is competitive *because* it wraps Polar/Paddle  
- Seamless setup of vendors we do not operate  

## Say

- Orbit sequences **your** sign / release / deploy  
- Keys and vendor accounts stay in the official UI  
- Payments and SaaS signup are out of scope unless we ship **our** cloud or **our** billing (we are not)
