# Payments setup path — optimize against vendor docs

**Status:** Slice A–C implemented (copy/Opens/docs) — L3 overlay dogfood still operator  
**Updated:** 2026-10-03  
**Owner:** Desktop Integrations wizards + website Solo checkout + optional host scripts  
**Parent:** [paddle-portal-minute-design](./paddle-portal-minute-design.md) · [website-paddle-checkout-design](./website-paddle-checkout-design.md) · [integrations-deploy-parity-design](./integrations-deploy-parity-design.md)

## Plan alignment

- **Handoff:** Maintainer dogfooding Paddle Solo; product value = script what can be scripted, then exact Open to vendor pages — not replace dashboards.
- **CANCELLED:** Vendor handoff coach (no form-fill theater).
- **Sources reviewed:** Paddle developer docs (quickstart, overlay, default payment link, client-side tokens, sandbox, digital products / fulfillment) via Paddle Docs MCP; Polar quickstart + sandbox docs; current Integrations copy + `seed-paddle-solo-sandbox.py`.

## Problem

Paddle’s **Get started** UI (Catalog → Pricing/checkout → Fulfillment → Test → Live) and Polar’s **product → checkout link** look similar at a glance. In practice Paddle has more **hard blockers** before an overlay works. Studio’s Paddle card is already closer to the right model (deep links, not the Get started wizard), but step order and copy still fight Polar-level simplicity and slightly disagree with our own seed script.

## Vendor truth (compressed)

### Polar (why it feels simple)

Official path: sandbox org → **Products** → **Checkout link** → paste URL → test card `4242…`. Optional API later. No default payment link. No separate client-side token vs API key for the happy path. Benefits/fulfillment can live in Polar.

Studio already mirrors this: few Opens, paste `PUBLIC_POLAR_*`.

### Paddle Billing (official quickstart)

| Official step | Hard requirement? | Dashboard / docs name | Studio today |
|---------------|-------------------|------------------------|--------------|
| Sandbox account (separate from live) | Yes | [sandbox signup](https://sandbox-vendors.paddle.com/signup) | Open ✓ |
| Catalog product + price (`pri_…`) | Yes | Catalog / products | Open + manual copy; seed script optional (docs only) |
| Client-side token (`test_…`) | Yes for Paddle.js | Authentication → **Client-side tokens** | Open ✓ |
| **Default payment link** | **Yes** — overlay fails with “Something went wrong” without it | Checkout → **Checkout configuration** | Open as “Checkout settings” ✓ |
| Pricing page + `Paddle.Checkout.open` | Yes for Solo site | Website host | Already wired on `/pricing` |
| Webhooks / fulfillment | Required for *auto* provisioning; not for first sandbox overlay | Events → Notifications; `transaction.completed` | Not linked; Solo uses manual `issue-license.py` |
| Go live | Later | Live account + website approval | Out of this slice |

Paddle also pushes **API key + MCP/skills** for catalog automation — same job as our seed script. Get started step **03 Fulfillment** is a SaaS tutorial default; for one-time digital goods, docs say you own fulfillment via `transaction.completed` (Billing ≠ Classic license delivery).

### Other payments (lighter note)

| Provider | Doc-shaped minimum | Studio gap |
|----------|-------------------|------------|
| Stripe | Products / Payment Links + keys | Opens OK; no Solo overlay job |
| Gumroad / Lemon | Product in vendor UI | Opens OK |
| Creem / Waffo | Dashboard + API keys on host | Opens OK; Put path exists |

Optimization effort concentrates on **Paddle** (highest friction, Solo processor). Do not bloat Polar/Stripe cards to match Paddle’s length.

## Gaps vs product intent

1. **Blurb lies by omission:** “Studio never creates products” — true in-app; false for the recommended host seed. Polar-simple path is the script; UI still teaches manual Catalog first.
2. **Seed path invisible in Desktop:** API key → `seed-paddle-solo-sandbox.py` lives in `PADDLE-SETUP.md` only. Dogfooders following Integrations do the long trail.
3. **Default payment link under-ranked:** Docs treat it as a pre-checkout gate; we bury it after Catalog + token. Failure mode is opaque.
4. **Get started / webhooks noise:** Opening Get started or requiring Notifications for Solo first paint fights “redirect to the page that matters.”
5. **Label drift:** Docs say **Checkout configuration**; we say Checkout settings (URL `/checkout-settings` is fine — label can match docs).
6. **Fulfillment honesty:** Solo license issue is operator/script, not Paddle Classic auto-keys. Don’t imply step 03 is required before overlay works.

## Target operator model (Paddle)

Two explicit lanes — same Opens, different amount of clicking:

```text
Lane A (preferred — Polar-feel)
  Open Authentication → create API key (product/price/client_token write)
  setx PADDLE_SANDBOX_API_KEY → python scripts/seed-paddle-solo-sandbox.py
  Open Checkout configuration → default payment link = http://localhost:4321
  pnpm website:dev → /pricing → test card
  (later) Orders for refunds · optional Events for webhooks · issue-license

Lane B (dashboard-only)
  Open Catalog → one-time product, copy pri_
  Open Authentication → Client-side tokens tab, copy test_
  Open Checkout configuration → default payment link
  Paste PUBLIC_PADDLE_* into apps/website/.env
  same test
```

Studio **never** holds `pdl_…`. Seed writes only gitignored `PUBLIC_*`. Dashboard remains source of truth for payment link, refunds, live cutover.

## Proposed product changes (next impl slice)

### A. Integrations Paddle copy + Opens (Desktop)

- Blurb: sandbox Opens + optional host seed; Studio does not create SKUs **in-app**.
- Steps: lead with Lane A one-liner; Lane B as fallback; put **default payment link** immediately after catalog/token (or before “paste env” / test) with “required or overlay fails.”
- Rename button label **Checkout settings** → **Checkout configuration** (URL unchanged).
- Optional tertiary Open: **Events / notifications** (fulfillment later) — not in the first five.
- Do **not** add Get started as a primary Open.
- Learn more may stay overlay tutorial; optional secondary docs URL = [quickstart](https://developer.paddle.com/get-started/quickstart) only if we add a second docs affordance (else leave as-is).

### B. Portal / shipctl strings

- Align human/portal step titles with “Checkout configuration” if they still say settings.
- Keep sandbox URLs; no live-first Opens.

### C. Docs

- `PADDLE-SETUP.md`: lead with Lane A; map Paddle Get started 01–04 → Studio Opens (skip 03 for Solo first paint; note manual license).
- Handoff / CURRENT: no change to product boundary.

### Out of scope this slice

- Auto-creating notification destinations or webhook receivers.
- Replacing Paddle Get started UI.
- Polar Solo processor revival.
- Creem/Waffo deep-link expansion beyond current Opens.

## Acceptance (when implemented)

- [x] Paddle wizard states Lane A (seed) before Lane B (manual).
- [x] Default payment link called out as required before `/pricing` works.
- [x] Checkout Open label matches Paddle docs (“configuration”).
- [x] Blurb/seed story consistent (no “never creates products” vs seed contradiction).
- [x] No Get started / webhook as setup blockers for Solo overlay.
- [ ] L3: maintainer completes sandbox overlay via Lane A using only Studio Opens + seed + `.env`.

## Proof

| Layer | Proof |
|-------|-------|
| L1 | Copy/URL review against this table |
| L2 | Desktop builds; openLinks hit sandbox 302→login |
| L3 | Seed + checkout settings + `/pricing` overlay + test card |
| L4 | Optional: one refund path via Orders Open |

## Decision

Ship the Desktop/docs alignment slice above as the next coding band after this design is accepted. Do not expand scope into webhook automation while Solo fulfillment remains operator `issue-license.py`.
