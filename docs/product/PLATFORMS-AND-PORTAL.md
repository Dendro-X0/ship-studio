# Platforms · Portal · Integrations — functionality & objectives

**Status:** Working tree (post–v0.2.3 UX arc) — **honest guide**, not finished product  
**Updated:** 2026-09-25  
**Audience:** Operators + agents continuing Desktop provider UX  
**Parents:** [PRODUCT.md](./PRODUCT.md) · [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)  
**Hosting improvement plan:** [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md)

```text
OBJECTIVE:  One place to pick a host / store / payment / email lane, open the vendor,
            finish human work, return to Publish → Confirm — without Studio holding secrets
            or pretending to deploy / notarize / charge cards.
NOT YET:    Bulletproof Desktop reliability · CDP dogfood · full provider parity ·
            one-click deploy · Studio-owned OAuth success detection
```

## Why this surface exists

Publish’s **inspection bay** (Self-sign · Official signing · Deploy) used to dead-end into Sign paths or “Use Local.” Operators need a **catalog** like Payments/Email: pick a lane, get a short checklist, open the real console, then continue the Publish spine.

Ship Studio remains a **portal and guide**. Vendor UIs stay authoritative.

## Host tiers (parity model)

| Tier | Hosts | Desktop Platforms | Portal / Publish |
|------|-------|-------------------|------------------|
| **A — Orbit deploy** | Cloudflare · Vercel · Netlify | Catalog + Portal Login CLI | Detect · OAuth · token · env Open≠Docs |
| **B — CLI host** | Fly · Railway | Catalog + Portal Login CLI | OAuth · (Railway tokens) · secrets/variables Docs |
| **C — Pages** | GitHub Pages | Catalog Open+Docs only | No Pages→PAT conflation; `github` portal for auth/CI |
| **D — Advanced host** | Render · DigitalOcean · Heroku · Amplify · Cloud Run · Azure SWA | **Not** in Platforms catalog yet | `shipctl portal --provider` · Advanced Publish `host.*` |
| **E — Local Orbit** | Orbit | Catalog · Ritual/Tools | No portal id |

Detail + slices: [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md).

## What works today (Desktop)

### Platforms (`platforms` view + sidebar)

| Group | Lanes | Primary actions |
|-------|-------|-----------------|
| **Hosting** | Orbit · Cloudflare · Vercel · Netlify · GitHub Pages · Fly · Railway | Open dashboard · Docs (when set) · optional Portal steps (when `provider` is a portal id) · Continue publishing · Use Local |
| **Official signing** | Apple · Microsoft · Google Play | Open vendor · checklist · Continue publishing |

- Shared chrome with Integrations: `provider-catalog.ts` (grid · sidebar · wizard aside).
- Probe CTAs: **Choose host** / **Choose platform** open this catalog (Hosting vs Official signing preference).
- Orbit has **no** Portal provider id — Open dashboard / Ritual deploy only (avoids `unknown provider 'orbit'`).
- Fly / Railway: Portal Login CLI (`fly auth login` / `railway login`) + kind-aware Docs (secrets / variables).
- GitHub Pages: **no** Portal steps — Open GitHub + Docs (Pages tutorial); PAT/`gh` auth stays on Portal → GitHub.
- Probe **Choose host** selects the detected host when known (wrangler → Cloudflare, etc.).
- Detect chips for wrangler / vercel / netlify / fly / railway / pages / orbit open Platforms on that card (github chip still opens Portal auth).

Specs: [platforms-catalog-design](../../specs/frontend/platforms-catalog-design.md) · [provider-wizard-setup-design](../../specs/frontend/provider-wizard-setup-design.md) · [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md)

### Integrations (Payments · Email)

| Group | Lanes | Minute loop |
|-------|-------|-------------|
| Payments | Polar · Stripe · Gumroad · Lemon · Paddle | Open → create listing → put `PUBLIC_*` / host secrets **outside** Studio → Continue publishing → Confirm |
| Email | Resend | Open API keys → Env Put on host → test on Resend → Confirm if Publish asks |

### Portal (`portal` view · `shipctl portal`)

| Behavior | Detail |
|----------|--------|
| Plan | Offline JSON steps per detected / filtered provider |
| **Open** | Vendor **settings** UI (`entry_url`) |
| **Docs** | Official tutorial (`docs_url`) — not a substitute for Open |
| **Login CLI** | Interactive terminal (`open_shipctl_terminal`) for OAuth rows only |
| Recover rows | Omitted when create URL ≡ token URL (no triple Cloudflare API-tokens links) |

Backend catalog: [provider-portal-design](../../specs/backend/provider-portal-design.md) · `crates/shipctl/src/portal.rs` · [hosting-portal-parity](../../specs/backend/hosting-portal-parity-design.md)

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
5. **CLI parity** — Desktop actions must map to `shipctl portal` / `env` / `publish` without inventing parallel state.

## Known shortcomings (do not patch in isolation)

These need a **reliability / product slice**, not drive-by UI edits:

| Gap | Why it hurts |
|-----|----------------|
| Desktop command reliability | **Slices 1–2 done** — Env Put terminal · unified `open_shipctl_terminal` · soft-fail taxonomy; next: user-load honesty — [desktop-reliability-design](../../specs/backend/desktop-reliability-design.md) |
| Provider catalog depth | Hosting list is a guide; Orbit/Fly/Railway/etc. lack full portal parity with Cloudflare/Vercel — see [hosting-portal-parity-investigation](../../specs/backend/hosting-portal-parity-investigation.md) · [design](../../specs/backend/hosting-portal-parity-design.md) |
| Stale / colliding Open·Docs | Netlify docs path moved; Fly/Railway dashboard Open repeated across steps — same design |
| GitHub Pages vs GitHub auth | **Slice 2 done** — Pages card is Open GitHub + Docs only; PAT stays on Portal → GitHub |
| Detect → highlight | **Slice 4 done** — Choose host / detect chips select preferred Platforms card (wrangler → Cloudflare) |
| Quiet Continue / outcome panel | Journey polish still thin after finish |
| CDP / CodaCtrl dogfood | Tauri needs remote debugging for live MCP client proof |
| Commerce E2E | Polar paid checkout deferred (`payment_ready`) |
| Env Put UX | Coach band **CANCELLED** — keep Env Put terminal; host account setup = vendor CLI / agents, not Studio onboarding theater — [vendor-handoff-coach-design](../../specs/backend/vendor-handoff-coach-design.md) |

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
| L1 | `cargo test -p shipctl portal::tests` · desktop `tsc` |
| L2 | Harbor Local: paced Continue · Platforms Orbit Open (no Portal steps) |
| L2 | Harbor Public: Cloudflare Portal — Open ≠ Docs; Login CLI opens terminal |

## Related specs (do not fork truth)

| Spec | Role |
|------|------|
| [hosting-portal-parity-design](../../specs/backend/hosting-portal-parity-design.md) | Host tiers · Open/Docs/Login matrix · slices 0–4 |
| [publish-journey-clarity-design](../../specs/frontend/publish-journey-clarity-design.md) | One green verb · paced Continue |
| [status-probe-ux-design](../../specs/frontend/status-probe-ux-design.md) | Inspection bay |
| [public-publish-ux-design](../../specs/frontend/public-publish-ux-design.md) | $29 bar · Stages primary |
| [verify-status-layers-design](../../specs/backend/verify-status-layers-design.md) | Disk · local CLI · operator CLI · human attest |
