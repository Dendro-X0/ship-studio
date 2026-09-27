# Launch choice board — design

**Status:** Proposed (next after panel redirects)  
**Updated:** 2026-09-26  
**Parents:** [launch-panel-redirects-design](./launch-panel-redirects-design.md) · [deployment-nav-design](./deployment-nav-design.md) · [platforms-catalog-design](./platforms-catalog-design.md)  
**Surfaces:** Desktop Launch · Sign · Deployment · Integrations · Env · Scopes

```text
GOAL: Launch is a short optional choice board — pick lanes, open dashboards, Confirm.
NOT: A 9–22 row checklist that re-lists work already owned by Sign / Deployment / Integrations.
```

## Problem (today)

Harbor Launch is already down to ~9 steps, but it still *feels* like a checklist:

| Row | Issue |
|-----|--------|
| doctor · configure · intent · flow · deploy | Auto noise next to human lanes |
| Open Sign / Open Scopes | Correct redirect, wrong chrome (looks like another PENDING chore) |
| legal.baseline | One-off file chore mixed into shipping lanes |
| Deploy — Orbit | Competing with **Deployment** panel |

Operators want **standardized options**: every path optional; choose providers inside dedicated dashboards — not “advance through every PENDING.”

## Product law

1. **Dashboards own vendor work.** Launch never lists Cloudflare, Polar, App Store, Signet substeps.
2. **Every lane is optional.** Skip = Confirm / Next with no fake Done on irreversible work; honesty stays on Confirm when the lane was opened.
3. **Detect suggests; human chooses.** Detection highlights recommended cards; user picks host / payment / stores inside the panel.
4. **Publish remains the minute spine.** Launch is the companion cockpit for *which* human surfaces to open.

## Dedicated dashboards (Ship + related)

| Dashboard | Owns | Provider choice |
|-----------|------|-----------------|
| **Sign** | Signet identity/build/release · official certs · store submit | Self vs official; Apple / Microsoft / Play when mobile/desktop in Scopes |
| **Deployment** | Host login · Put · landing · DB/BaaS | Orbit / CF / Vercel / Netlify / Fly / Railway / Pages — **pick one primary** |
| **Integrations** | Payments · email | Polar / Stripe / … — **only show when lane enabled**; default off |
| **Env / tokens** | Put secrets sprint | Driven by chosen host + detected empty secrets |
| **Scopes** | What ships (Web / API / Desktop) | Checkbox surface set (already) |
| **Legal** *(new thin panel or Dashboard card)* | LICENSE · SECURITY · TRUST | Optional checklist — not a Launch PENDING forever |

**Do not add** more Ship nav items without collapsing something else. Prefer Legal as a card on Dashboard / Sign, not a 6th primary nav.

## Launch UI target — choice board

Replace the long `portal-steps` list with **three bands**:

### A — Prep (auto, collapsed)

Silent or one “Prep · N ok” chip: doctor · configure · intent. No per-row Confirm theater.

### B — Lanes (optional cards)

Each card = one dashboard. CTA = **Open …**. Secondary = Skip.

| Card | Shown when | Primary |
|------|------------|---------|
| **Targets** | Always (if multi-scope) | Open Scopes |
| **Sign** | Desktop / mobile / Signet detected *or* user enables | Open Sign |
| **Deployment** | Web/API host detected *or* user enables | Open Deployment |
| **Payments** | User enables **or** Polar/Stripe/… already in repo | Open Integrations |
| **Env** | Put queue non-empty after host choice | Open Env |
| **Legal** | Missing LICENSE/SECURITY | Open (Dashboard/Sign legal strip) |

Detection lights a **Suggested** badge; empty apps get fewer cards.

### C — Ship cut (optional run)

One row max: **Dry-run** and/or **Deploy** (Orbit/local CLI) — only if Deployment lane was chosen and intent is Public. Otherwise omit.

## Provider choice lives inside panels

| Panel | Choice UX |
|-------|-----------|
| Deployment | Catalog grid (already) — select **primary host**; Login CLI / Put for that host only |
| Integrations | Existing wizards — enable Payments lane first; no Launch row if lane off |
| Sign | Paths list + probe (already) — official lanes appear from Scopes / Signet |

Launch never asks “login Cloudflare *and* Vercel *and* Netlify.”

## Mapping from current Launch ids

| Today | Tomorrow |
|-------|----------|
| `doctor` · `configure` · `intent` | Band A (collapsed prep) |
| `scopes` | Lane · Targets |
| `legal.baseline` | Lane · Legal (or Dashboard) |
| `sign.panel` | Lane · Sign |
| `oauth.hosts` · `deploy.panel` | Lane · Deployment |
| `integrations.panel` | Lane · Payments (opt-in) |
| `env.sprint` | Lane · Env (after host) |
| `flow_dry_run` · `deploy` | Band C (optional) |
| `listing.packages` | Portal / Deploy advanced — not Launch |

## Implementation slices

| Slice | Work | Proof |
|-------|------|--------|
| **L0** | This design + handoff queue | Spec only |
| **L1** | Launch view chrome: Band A/B/C layout (Desktop); keep shipctl step ids, regroup in UI | Harbor shows ≤6 visible lane cards |
| **L2** | shipctl: emit `lane` + `optional` + `suggested` on steps; drop flow/deploy from Launch when Local | `cargo test -p shipctl` |
| **L3** | Payments lane opt-in (studio.json / UI toggle); Integrations hidden until on | Harbor no Payments card by default |
| **L4** | Deployment “primary host” remembered; oauth.hosts verify that host only | Verify uses chosen provider |

## Non-goals

- Coach / floating overlay ([CANCELLED](../backend/vendor-handoff-coach-design.md))
- Fake one-click deploy or Studio-held OAuth tokens
- Replacing Publish’s Confirm honesty spine
- Expanding Tier D hosts into Deployment until asked

## Proof (when L1 ships)

- L1: desktop `tsc` · Harbor Launch ≤6 lane cards + collapsed Prep  
- L2: No Integrations card without payments detect **or** opt-in  
- L3: Open Sign / Open Deployment still land on existing panels  

## Decision for maintainer

Adopt **choice board** (Bands A/B/C) as the Launch north star. Next code slice = **L1** (UI regroup only) unless S1.1 GIF recording must finish first on the current 9-step list.
