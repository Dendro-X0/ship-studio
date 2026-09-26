# Ship Studio — improvement backlog

**Status:** Living idea list — **not** an activate queue  
**Updated:** 2026-09-23  
**Cadence:** Pick from here when energy allows; hub Adaptive stays idle unless a real ship needs a lane  
**Canon:** [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · [PRODUCT.md](./PRODUCT.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md) · [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
**Strategy sim:** `strategy-research-lab/strategies/sims/S-SHIP-STUDIO-hub.md`

```text
Value bar:  Multi-repo / multi-surface final-mile order + honest Verify + Continue scripts
            (portal/guide — not vendor replacement; utility > SaaS starter kits)
Not:        Hype · chatbot-substitutable checklists · fake auto-publish · replace Polar/CF/stores
Price:      $29 one-time (Polar) — paid delta = fewer missed gates across many projects
Role:       DUAL TRUNK (2026-09-25) — long-term offering + client portfolio face
            Depth shelf: Obscur · Vectis · Hobby gifts stay separate
            Intent: strategy-research-lab/strategies/ship-studio-dual-trunk.md
```

Every candidate should still answer: Does it reduce missed gates for multi-surface ships? Does it stay inside scope (no vendor replacement, no dangerous live publish)? Would a stranger with many OSS repos feel this was worth $29 — or only a pasted README?

---

## Priority tags

| Tag | Meaning |
|-----|---------|
| **P0** | Stranger buy/download loop or trust/positioning blockers |
| **P1** | Clarity / paid delta / dogfood proof |
| **P2** | Depth when a real ship or pulse asks |
| **Reject** | Out of scope or hollow value |

---

## P0 — Complete the commercial loop

| ID | Idea | Notes |
|----|------|-------|
| S0.1 | **W4 — license delivery** | Built; live email dogfood when Polar charge/free checkout works |
| S0.2 | **Refund path dogfood** | Built; E2E deferred on Polar `payment_ready` |
| S0.3 | **Public download path** | Done — [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) installer + zip (v0.2.0 / v0.1.0 retained) |
| S0.4 | **Paid delta on `/pricing`** | Done (`28f696e`) |
| S0.5 | **Brand disambiguation** | Done (`28f696e`) |
| S0.6 | **Polar env wizard → deploy** | Checklist already in Desktop Integrations; one dogfood that `PUBLIC_POLAR_*` lands on website deploy without Studio writing secrets |
| S0.7 | **Windows installer** | Done — NSIS on [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) |
| S0.8 | **In-app update check** | Next after demo GIFs — notice newer GitHub Releases; no silent force-install; human Confirm to open download |

---

## P1 — Perception of real value (anti-hype)

| ID | Idea | Notes |
|----|------|-------|
| S1.1 | **Demo shelf = multi-target tedium** | GIF/script on **v0.2.3** — bind **`fixtures/harbor`** (Desktop+Docs), `scripts/harbor-reset` — [demo-subject-design](../../specs/frontend/demo-subject-design.md) · [SCRIPT](../assets/demo/v0.2.1/SCRIPT.md) |
| S1.0a | **Desktop nav freeze** | Hot-path + silent spawn + paint-before-shipctl + output mirror truncate shipped in **v0.2.1** |
| S1.2 | **Dashboard honesty polish** | Surface Signet ok / Orbit missing, dirty git, step N/M without fake “all shipped” |
| S1.2a | **Publish progress clarity** | Done · required · optional/later bands + summary strip — [publish-progress-clarity-design](../../specs/frontend/publish-progress-clarity-design.md) · **shipped Desktop** |
| S1.2b | **Workflow stage cards + pager** | Dashboard presets → linear Publish stages — [workflow-stages-design](../../specs/frontend/workflow-stages-design.md) · **Desktop slice 1 wired** (L2 dogfood pending) |
| S1.2c | **Public Publish UX ($29 bar)** | Single primary · rail diet · inline Scopes · copy pass — [public-publish-ux-design](../../specs/frontend/public-publish-ux-design.md) · **A+B+C Desktop** (D/E/F pending) |
| S1.2j | **Hosting portal parity** | Tier A–E · Fly/Railway · Pages≠PAT · detect→highlight — [design](../../specs/backend/hosting-portal-parity-design.md) · **slices 0–4 done** |
| S1.2i | **Platforms · Portal product doc** | Functionality · objectives · known gaps — [PLATFORMS-AND-PORTAL](./PLATFORMS-AND-PORTAL.md) · **docs shipped** |
| S1.2k | **Desktop reliability (TTY · toasts)** | Env Put · unified opener · soft taxonomy — [design](../../specs/backend/desktop-reliability-design.md) · **slices 1–2**; slice 3 next |
| S1.2h | **Desktop silent failures** | Login CLI terminal · portal provider guard · actionable fail toasts — [desktop-silent-failures-investigation](../../specs/backend/desktop-silent-failures-investigation.md) · **Desktop slice 1**; continued as S1.2k |
| S1.2g | **Provider wizard setup** | Hosting/Payments/Email minute loop + Continue publishing — [provider-wizard-setup-design](../../specs/frontend/provider-wizard-setup-design.md) · **Desktop slice 1** |
| S1.2f | **Platforms catalog** | Hosting + Official signing picker (shared with Integrations chrome) — [platforms-catalog-design](../../specs/frontend/platforms-catalog-design.md) · **Desktop slice 1** |
| S1.2e | **Publish journey clarity** | One green verb · paced Continue (~2s) · scrubber — [publish-journey-clarity-design](../../specs/frontend/publish-journey-clarity-design.md) · **Desktop slices 1–3** |
| S1.2d | **Status probe (Sign · Deploy)** | Inspection bay: icons · badges · suggestions · CTAs — [status-probe-ux-design](../../specs/frontend/status-probe-ux-design.md) · **Desktop slice 2** |
| S1.3 | **Never-say block on site + README** | No “one-click deploy” · no “we silence SmartScreen” · no “replaces Cloudflare/Polar” · Studio does not write secrets |
| S1.4 | **One intent page** | “Multi-surface final-mile vs CI-only / checklist” — SEO without belonging theater |
| S1.5 | **OPERATOR-NEXT as product feature** | Market human gates as honesty, not unfinished bugs |
| S1.6 | **Cafe / FAQ stub** | SmartScreen? vs other Ship Studio? Why $29? Signet required? |
| S1.7 | **Offline badge meaning** | Done — topbar Offline title + PRODUCT/SCOPE status layers ([verify-status-layers-design](../../specs/backend/verify-status-layers-design.md)) |

---

## P1 — Hub UX (Integrations / Publish)

| ID | Idea | Notes |
|----|------|-------|
| S1.10 | **Integrations → Publish handoff** | From Polar wizard, clear “Continue publishing” with next plan step — less sidebar archaeology |
| S1.11 | **Wizard completion Confirm** | After Open dashboard + paste env, explicit Confirm so `.ship/publish.json` advances |
| S1.12 | **Provider “done” criteria** | Per Polar/Stripe/…: what Verify checks (URL present vs webhook live) — stay honest · pairs with [verify-status-layers-design](../../specs/backend/verify-status-layers-design.md) |
| S1.13 | **Resend / email wizard parity** | Same Open → checklist → Confirm pattern as Polar |
| S1.14 | **Local vs Public intent copy** | Make Local/Public toggle consequences one sentence each on Dashboard |
| S1.15 | **Dirty-tree Publish cue** | Dirty-on-main warnings: optional soft gate before release steps |
| S1.17 | **Multi-project portfolio cue** | Recents / suite-oriented “next repo to ship” for 30+ OSS indies — stay local, no cloud hub |
| S1.18 | **Web3 / chain release lanes** | Only when a real ship asks — detect + Open official explorers/wallets; no custody or auto-broadcast |

---

## P2 — Depth (only on demand)

| ID | Idea | Notes |
|----|------|-------|
| S2.1 | Release-surface **Db** doctor/scopes gaps | Per [release-surface-map.md](../../specs/backend/release-surface-map.md) — when a ship needs it |
| S2.2 | Mobile env / listing catalog depth | Still no store API upload |
| S2.3 | Container live-check | Push stays Confirm |
| S2.4 | MCP `ship_*` depth | For agent operators; secondary ICP |
| S2.5 | Graduate / Authenticode notes Run depth | Still human finishes certs |
| S2.6 | Suite URL sync dogfood | Sibling projects after marketing.deploy |
| S2.7 | Watch mode cues | Publish watch prompts when Verify ready — tune noise |
| S2.8 | Clavis vault export discoverability | Optional backup — not password-manager pitch |
| S2.9 | Annual / Pro tier | Only after $29 Solo pulse + written soft-no |
| S2.10 | Tip / OSS sponsorship footer | Optional; don’t confuse with license |

---

## Reject (do not backlog as “improvements”)

| Item | Why |
|------|-----|
| Auto `docker push` / live npm/cargo publish / `gh release create` | Dangerous automation — scope kill |
| Replace Polar / Cloudflare / Apple / store UIs | Out of scope; hollow “platform” |
| Cloud multi-tenant portfolio SaaS | Scope non-goal v0 |
| AI “ship agent” that claims to finish OAuth | Chatbot theater; scam-prior |
| Bought reviews / hype launch | Operator rules |
| Paywall core sequencing with no OSS path | Ethics + conversion trust |
| Merge Unstick / Anti-SE / Assess into Studio | Wrong product; keep adapters thin |
| k8s controllers / mobile store API upload | Explicitly deferred / cancelled |
| Endless payment logos before Polar E2E | Breadth without proof |

---

## Suggested order (post v0.2.3)

```text
1. S1.1       Live silent demo GIFs on v0.2.3 (Continue + progress bands)
2. Deploy site · Polar URLs · PUBLIC_DOWNLOAD_URL → v0.2.3
3. S0.8       In-app update check (GitHub Releases notice · Confirm open)
4. S0.1–S0.2  Live buy/refund only when Polar payment_ready (not a release blocker)
```

Earlier default order (commerce-first) stays valid after Polar unlocks; do not block the week on Pay now.
---

## Related

- [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)  
- [../CURRENT.md](../CURRENT.md) · [../handoffs/current-session.md](../handoffs/current-session.md)  
- [../../specs/backend/shipping-hub-north-star.md](../../specs/backend/shipping-hub-north-star.md)  
- [../../specs/backend/release-surface-map.md](../../specs/backend/release-surface-map.md)  
- [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
