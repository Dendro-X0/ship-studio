# Platforms · Portal · Integrations — functionality & objectives

**Status:** Shipped guide surfaces on **v0.2.3** (+ Launch Payments opt-in · primary host) — honest portal, not vendor replacement  
**Updated:** 2026-10-03  
**Audience:** Operators + agents continuing Desktop provider UX  
**Parents:** [PRODUCT.md](./PRODUCT.md) · [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)  
**Hosting improvement plan:** [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md)

```text
OBJECTIVE:  Ops / deploy — pick a **host** (and signing), Login CLI / Put deploy env,
            return to Publish → Confirm. **Not** a Polar/Paddle/email integrations product.
SHIPPED:    Platforms catalog · Portal Open/Login CLI · host deploy lanes (historical Payments UI leftover)
NOT YET:    Subtract Integrations Payments/Email surface ([ops-deploy-pivot](../../specs/backend/ops-deploy-pivot-design.md))
CANCELLED:  Third-party integration setup as a Studio job
```

## Two different “payments” jobs

| Job | Surface | Ready? |
|-----|---------|--------|
| **Bookmark Polar / Stripe / … dashboards** | Desktop **Integrations** → Open vendor (optional). **Setup happens in the vendor UI.** Studio does not complete Polar/Paddle onboarding. | **Thin** — expansion **PAUSED**. Maintainer Polar worked **without** Studio. |
| **Sell Orbit Yard Solo on the official website** | Optional Buy — **not** Paddle dogfood ([without Paddle](../../specs/backend/solo-commerce-without-paddle-design.md)) | **Optional** — OSS download is enough |

Do not conflate them. Integrations is for *their* commerce; website Buy is for *this* product’s license.

## Why this surface exists

Publish’s **inspection bay** (Self-sign · Official signing · Deploy) used to dead-end into Sign paths or “Use Local.” Operators need a **catalog** like Payments/Email: pick a lane, get a short checklist, open the real console, then continue the Publish spine.

Orbit Yard remains a **portal and guide**. Vendor UIs stay authoritative.

**Structure before proceed:** Deployment / Sign wizards show a structure banner. Hard Deploy lanes (Self-host · Umami · Flowsint · Medusa · Cloudflare · Vercel · Netlify · Fly · Railway) block spawn when the bound folder does not match detect. Soft notes (Pages · Orbit docs · Sign Open) warn without blocking Open. Spec: [structure-proceed-gate-design](../../specs/backend/structure-proceed-gate-design.md).

## Host tiers (parity model)

| Tier | Hosts | Desktop Platforms | Portal / Publish |
|------|-------|-------------------|------------------|
| **A — Orbit deploy** | Cloudflare · Vercel · Netlify | Catalog + Portal Login CLI · Import env / Deploy all-in-one · live URL toast (P1) | Detect · OAuth · token · env Open≠Docs · Put CLI · Vercel `--project` name |
| **B — CLI host** | Fly · Railway | Catalog + Portal Login CLI · **Import env / Put** · **Deploy** · Troubleshoot bay (P1) | OAuth · secrets/variables via CLI stdin · `[link]` gate · Fly URL from `app =` |
| **C — Pages** | GitHub Pages | Catalog Open+Docs only | No Pages→PAT conflation; `github` portal for auth/CI |
| **D — Advanced host** | Render · DigitalOcean · Heroku · Amplify · Cloud Run · Azure SWA | **Not** in Platforms catalog yet | `orbityard portal --provider` · Advanced Publish `host.*` |
| **E — Local Orbit** | Orbit | Catalog · Ritual/Tools | No portal id |

Detail + slices: [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md).

## What works today (Desktop)

### Platforms (`platforms` view + sidebar)

| Group | Lanes | Primary actions |
|-------|-------|-----------------|
| **Hosting** | Orbit · Cloudflare · Vercel · Netlify · GitHub Pages · Fly · Railway | Open dashboard · Docs (when set) · optional Portal steps (when `provider` is a portal id) · Continue publishing · Use Local |
| **Official signing** | Apple · Microsoft · Google Play | Open vendor · checklist · Continue publishing |

- Shared chrome with Integrations: `provider-catalog.ts` (grid · wizard aside).
- Probe CTAs: **Choose host** / **Choose platform** open this catalog (Hosting vs Official signing preference).
- Orbit has **no** Portal provider id — Open dashboard / Ritual deploy only (avoids `unknown provider 'orbit'`).
- Fly / Railway: Portal Login CLI + **Deploy** (hostdeploy after human `fly launch` / `railway link`) + **Import env…** / Put — values never in `.ship/`; `[link]` if config missing.
- GitHub Pages: **no** Portal steps — Open GitHub + Docs (Pages tutorial); PAT/`gh` auth stays on Portal → GitHub.
- Probe **Choose host** selects the detected host when known (wrangler → Cloudflare, etc.).
- Detect chips for wrangler / vercel / netlify / fly / railway / pages / orbit open Platforms on that card (github chip still opens Portal auth).

Specs: [platforms-catalog-design](../../specs/frontend/platforms-catalog-design.md) · [provider-wizard-setup-design](../../specs/frontend/provider-wizard-setup-design.md) · [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md)

### Integrations (Payments · Email)

| Group | Lanes | Minute loop |
|-------|-------|-------------|
| Payments | Polar · Stripe · Gumroad · Lemon · Paddle · Creem · Waffo | Open → create listing → put `PUBLIC_*` / host secrets **outside** Studio → Continue publishing → Confirm |
| Email | Resend | Open API keys → Env Put on host → test on Resend → Confirm if Publish asks |

### Portal (`portal` view · `orbityard portal`)

| Behavior | Detail |
|----------|--------|
| Plan | Offline JSON steps per detected / filtered provider |
| **Open** | Vendor **settings** UI (`entry_url`) |
| **Docs** | Official tutorial (`docs_url`) — not a substitute for Open |
| **Login CLI** | Interactive terminal (`open_orbityard_terminal`) for OAuth rows only |
| Recover rows | Omitted when create URL ≡ token URL (no triple Cloudflare API-tokens links) |

Backend catalog: [provider-portal-design](../../specs/backend/provider-portal-design.md) · `crates/orbityard/src/portal.rs` · [hosting-portal-parity](../../specs/backend/hosting-portal-parity-design.md)

### Supporting Desktop behaviors (same arc)

| Piece | Role |
|-------|------|
| Status probe | Sign / Publish inspection bay — readiness without fake Done |
| Paced Continue | Auto gates advance one-at-a-time (~2s dwell + scrubber) |
| Collapsible nav | Ship · Targets · Platforms · Integrations · More · Run |
| Fail toasts | Soft failures include stderr; sticky FAILED reserved for harder crashes |
| Harbor fixture | `fixtures/harbor` — demo subject for Local/Public dogfood |

## Objectives (north star)

1. **One green path** — Publish spine owns Confirm; Platforms / Integrations / Portal are detail panels with **Continue publishing**.
2. **Open ≠ Docs** — settings UI vs official tutorials stay separate actions.
3. **Honesty** — no Studio-held secrets; no auto-Done on Human/OAuth/deploy; Local can omit hosted Live check.
4. **Shared picker** — any future “pick a provider” panel reuses the catalog utility.
5. **CLI parity** — Desktop actions must map to `orbityard portal` / `env` / `publish` without inventing parallel state.

## Known shortcomings (do not patch in isolation)

These need a **reliability / product slice**, not drive-by UI edits:

| Gap | Why it hurts |
|-----|----------------|
| Desktop command reliability | **Slices 0–4 done** — Env Put terminal · unified opener · soft-fail · user-load honesty · Ritual N-class — [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md) |
| Provider catalog depth | Hosting list is a guide; Orbit/Fly/Railway/etc. lack full portal parity with Cloudflare/Vercel — see [hosting-portal-parity-investigation](../../specs/backend/hosting-portal-parity-investigation.md) · [design](../../specs/backend/hosting-portal-parity-design.md) |
| Stale / colliding Open·Docs | Netlify docs path moved; Fly/Railway dashboard Open repeated across steps — same design |
| GitHub Pages vs GitHub auth | **Slice 2 done** — Pages card is Open GitHub + Docs only; PAT stays on Portal → GitHub |
| Detect → highlight | **Slice 4 done** — Choose host / detect chips select preferred Platforms card (wrangler → Cloudflare) |
| Quiet Continue / outcome panel | Journey polish still thin after finish |
| CDP / CodaCtrl dogfood | Tauri needs remote debugging for live MCP client proof |
| Commerce E2E | **Studio Solo** Polar checkout deferred (`payment_ready`) — distinct from Integrations Payments **guide** |
| Env Put UX | **Overhaul O0–O5 done** — Portal + Platforms Put primary; Harbor proof — [overhaul](../../specs/backend/ship-studio-overhaul-design.md) · [evidence](../handoffs/evidence-harbor-client-honesty.md); coach **CANCELLED** |

Investigation seed: [desktop-silent-failures-investigation](../../specs/backend/desktop-silent-failures-investigation.md) · full band: [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md)

## Operator minute loop (canonical)

```text
Dashboard workflow card → Publish checkpoint
  → (optional) Platforms / Integrations pick lane
  → Open settings · Docs if stuck · Login CLI only for OAuth
  → Put secrets on host (Env / wrangler secret / vendor UI)
  → Continue publishing → Confirm / Live check URL
```

## Proof pointers

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p orbityard portal::tests` · desktop `tsc` |
| L2 | Harbor Local: paced Continue · Platforms Orbit Open (no Portal steps) |
| L2 | Harbor Desktop dogfood — Publish copy · Payments toggle · primary host — [evidence](../handoffs/evidence-harbor-desktop-dogfood.md) |

## Related specs (do not fork truth)

| Spec | Role |
|------|------|
| [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md) | Host tiers · Open/Docs/Login matrix · slices 0–4 |
| [publish-journey-clarity-design](../../specs/frontend/publish-journey-clarity-design.md) | One green verb · paced Continue |
| [status-probe-ux-design](../../specs/frontend/status-probe-ux-design.md) | Inspection bay |
| [public-publish-ux-design](../../specs/frontend/public-publish-ux-design.md) | $29 bar · Stages primary |
| [verify-status-layers-design](../../specs/backend/verify-status-layers-design.md) | Disk · local CLI · operator CLI · human attest |
