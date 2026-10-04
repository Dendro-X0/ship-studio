# Ship Studio — improvement backlog

**Status:** Living idea list — through **S1.6 FAQ**; **close-cut** = OSS done, not Paddle sale  
**Updated:** 2026-10-04  
**Cadence:** Close-cut then freeze · no portal expansion · Paddle overlay optional · activate one depth band at a time  
**Canon:** [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · [PRODUCT.md](./PRODUCT.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md) · [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md) · [ship-studio-overhaul-design](../../specs/backend/ship-studio-overhaul-design.md)  
**Strategy sim:** `strategy-research-lab/strategies/sims/S-SHIP-STUDIO-hub.md`

```text
Value bar:  Finishable host cut — stream process · show deployed URL · open provider
            dashboard on the result · troubleshoot failures ($29) — plus multi-repo
            final-mile order + honest Verify (portal/guide; Put/Login not Docs)
Not:        Hype · encyclopedia dumps · CDP Create-Token form-fill · fake auto-publish ·
            coach theater · Studio-held secrets
Price:      $29 one-time (Paddle) — paid delta = account-holders ship without DevOps fluency
Role:       DUAL TRUNK (2026-09-25) — long-term offering + client portfolio face
            Depth shelf: Obscur · Vectis · Hobby gifts stay separate
            Intent: strategy-research-lab/strategies/ship-studio-dual-trunk.md
```

Every candidate should still answer: Does it reduce missed gates for multi-surface ships? Does it stay inside scope (no vendor replacement, no dangerous live publish)? Would a stranger with many OSS repos — or a founder who owns vendor accounts but not DevOps fluency — feel this was worth $29 — or only a pasted README?

**Active $29 band:** [hosted-deploy-ops-value-bar-design](../../specs/backend/hosted-deploy-ops-value-bar-design.md) (**H1–H5 shipped** — closed).

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
| **S1.2m** | **Client honesty overhaul** | **Done** O0–O5 — [overhaul](../../specs/backend/ship-studio-overhaul-design.md) · [Harbor evidence](../handoffs/evidence-harbor-client-honesty.md) |
| S0.1 | **W4 — license delivery** | Built; live email dogfood when a Paddle sandbox overlay succeeds |
| S0.2 | **Refund path dogfood** | Built; E2E deferred on Polar `payment_ready` |
| S0.3 | **Public download path** | Done — [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) installer + zip (v0.2.0 / v0.1.0 retained) |
| S0.4 | **Paid delta on `/pricing`** | Done (`28f696e`) |
| S0.5 | **Brand disambiguation** | Done (`28f696e`) |
| S0.6 | **Polar env wizard → deploy** | Checklist already in Desktop Integrations; one dogfood that `PUBLIC_POLAR_*` lands on website deploy without Studio writing secrets |
| S0.7 | **Windows installer** | Done — NSIS on [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) |
| S0.8 | **In-app update check** | **Done** — notice newer GitHub Releases; Confirm **Open download**; no silent install — [design](../../specs/frontend/in-app-update-check-design.md) |

---

## P1 — Perception of real value (anti-hype)

| ID | Idea | Notes |
|----|------|-------|
| **S1.2n** | **Hosted deploy ops ($29 bar)** | Stream CLI deploy · results · dashboard · troubleshoot · MCP playbook — [design](../../specs/backend/hosted-deploy-ops-value-bar-design.md) · **H1–H5 shipped** |
| S1.1 | **Demo shelf = multi-target tedium** | **Done** — T1–T7 live on `/demo` (`v0.2.1`, 720×420, mirrored) — [SCRIPT](../assets/demo/v0.2.1/SCRIPT.md) · [demo-subject-design](../../specs/frontend/demo-subject-design.md) |
| S1.0a | **Desktop nav freeze** | Hot-path + silent spawn + paint-before-shipctl + output mirror truncate shipped in **v0.2.1** |
| S1.2 | **Dashboard honesty polish** | **Slice 1 done** — General health tiles · Signet/Orbit/dirty/step N/M · no “Shipped” — [design](../../specs/frontend/dashboard-honesty-design.md) |
| S1.2a | **Publish progress clarity** | Done · required · optional/later bands + summary strip — [publish-progress-clarity-design](../../specs/frontend/publish-progress-clarity-design.md) · **shipped Desktop** |
| S1.2b | **Workflow stage cards + pager** | Dashboard presets → linear Publish stages — [workflow-stages-design](../../specs/frontend/workflow-stages-design.md) · **Desktop slice 1 wired** (L2 dogfood pending) |
| S1.2c | **Public Publish UX ($29 bar)** | **Done A–F** — single primary · rail · inline Scopes · copy · workflow preview · General nav — [design](../../specs/frontend/public-publish-ux-design.md) |
| S1.2j | **Hosting portal parity** | Tier A–E · Fly/Railway · Pages≠PAT · detect→highlight — [design](../../specs/backend/hosting-portal-parity-design.md) · **slices 0–4 done** |
| S1.2i | **Platforms · Portal product doc** | Functionality · objectives · known gaps — [PLATFORMS-AND-PORTAL](./PLATFORMS-AND-PORTAL.md) · **docs shipped** |
| S1.2k | **Desktop reliability (TTY · toasts)** | Env Put · unified opener · soft taxonomy · user load honesty · Ritual N-class — [design](../../specs/backend/desktop-reliability-design.md) · **slices 0–4 done**; Later parked |
| S1.2l | **Vendor handoff coach** | **CANCELLED** — in-app coach subtracted; prefer official CLI / agents for host setup — [design](../../specs/backend/vendor-handoff-coach-design.md) |
| S1.2h | **Desktop silent failures** | Login CLI terminal · portal provider guard · actionable fail toasts — [desktop-silent-failures-investigation](../../specs/backend/desktop-silent-failures-investigation.md) · **Desktop slice 1**; continued as S1.2k |
| S1.2g | **Provider wizard setup** | Hosting/Payments/Email minute loop + Continue publishing — [provider-wizard-setup-design](../../specs/frontend/provider-wizard-setup-design.md) · **Desktop slice 1** |
| S1.2f | **Platforms catalog** | Hosting + Official signing picker (shared with Integrations chrome) — [platforms-catalog-design](../../specs/frontend/platforms-catalog-design.md) · **Desktop slice 1** |
| S1.2e | **Publish journey clarity** | One green verb · paced Continue (~2s) · scrubber — [publish-journey-clarity-design](../../specs/frontend/publish-journey-clarity-design.md) · **Desktop slices 1–3** |
| S1.2d | **Status probe (Sign · Deploy)** | Inspection bay: icons · badges · suggestions · CTAs — [status-probe-ux-design](../../specs/frontend/status-probe-ux-design.md) · **Desktop slice 2** |
| S1.3 | **Never-say block on site + README** | **Done** — ban table + audit; T7 `/demo` caption fixed — [design](../../specs/frontend/never-say-block-design.md) |
| S1.4 | **One intent page** | **Done** — `/intent` final-mile vs CI-only / checklist · nav + home link — [design](../../specs/frontend/one-intent-page-design.md) |
| S1.5 | **OPERATOR-NEXT as product feature** | **Done** — `/honesty` + docs reframe; gates ≠ backlog — [design](../../specs/frontend/operator-next-product-design.md) |
| S1.6 | **Cafe / FAQ stub** | **Done** — `/faq` SmartScreen · brand · Solo price · Signet — [design](../../specs/frontend/cafe-faq-design.md) |
| S1.7 | **Offline badge meaning** | Done — topbar Offline title + PRODUCT/SCOPE status layers ([verify-status-layers-design](../../specs/backend/verify-status-layers-design.md)) |

---

## P1 — Hub UX (Integrations / Publish)

| ID | Idea | Notes |
|----|------|-------|
| **S1.19** | **Ship nav shell consistency** | **Done** — Targets + Integrations are single Ship items (Sign/Deployment pattern). Spec: [integrations-nav-design](../../specs/frontend/integrations-nav-design.md) |
| S1.10 | **Integrations → Publish handoff** | **Done** — Continue publishing names/focuses `listing.*`; commerce Related → Integrations — [design](../../specs/frontend/integrations-publish-handoff-design.md) |
| S1.11 | **Wizard completion Confirm** | **Done** — **Confirm gate** on Integrations / Deployment / Sign when matching Publish step is current — [design](../../specs/frontend/wizard-completion-confirm-design.md) |
| S1.12 | **Provider “done” criteria** | **Slice 1 done** — wizard **Done when…** lines (human attest) — [design](../../specs/frontend/provider-done-criteria-design.md) |
| S1.13 | **Resend / email wizard parity** | **Done** — Put RESEND_API_KEY · Confirm/Continue → `env.sprint` — [design](../../specs/frontend/resend-wizard-parity-design.md) |
| S1.14 | **Local vs Public intent copy** | **Done** — Dashboard `#intent-cue` + topbar titles (one sentence each) |
| S1.15 | **Dirty-tree Publish cue** | **Done** — soft Confirm warn on deploy/listing/release · Publish strip cue — [design](../../specs/frontend/dirty-tree-publish-cue-design.md) |
| S1.16 | **Targets catalog depth** | **Done** — Folder · Deploy · auto-detect on bind · dirty Save cue — [design](../../specs/frontend/targets-catalog-parity-design.md) |
| S1.17 | **Multi-project portfolio cue** | Recents / suite-oriented “next repo to ship” for 30+ OSS indies — stay local, no cloud hub |
| S1.18 | **Web3 / chain release lanes** | Only when a real ship asks — detect + Open official explorers/wallets; no custody or auto-broadcast |
| S1.21 | **Creem / Waffo / Paddle guide depth** | **Done** — detect · listing · portal · Desktop wizards · site honesty (Solo stays Polar) — [design](../../specs/backend/commerce-creem-waffo-design.md) |

---

## P2 — Depth (only on demand)

| ID | Idea | Notes |
|----|------|-------|
| S2.1 | Release-surface **Db** doctor/scopes gaps | Per [release-surface-map.md](../../specs/backend/release-surface-map.md) — when a ship needs it |
| S2.2 | Mobile env / listing catalog depth | Still no store API upload |
| S2.3 | Container live-check | Push stays Confirm |
| S2.4 | MCP `ship_*` depth | Contract + gaps G1–G6 — [mcp-assist-contract-design](../../specs/backend/mcp-assist-contract-design.md); implement wrappers when a real agent ship needs them |
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
| Vendor handoff coach / in-app encyclopedia dumps | **CANCELLED** — [vendor-handoff-coach-design](../../specs/backend/vendor-handoff-coach-design.md); use overhaul Put/Login CTAs instead |
| Bought reviews / hype launch | Operator rules |
| Paywall core sequencing with no OSS path | Ethics + conversion trust |
| Merge Unstick / Anti-SE / Assess into Studio | Wrong product; keep adapters thin |
| k8s controllers / mobile store API upload | Explicitly deferred / cancelled |
| Endless payment logos before Polar E2E | Breadth without proof |

---

## Suggested order (post S1.6)

```text
Near
1. Optional   Recompress/spot-check demo GIFs if weight/noise bothers

Commerce / site (not a release blocker)
2. Deploy site · PUBLIC_PADDLE_* sandbox overlay · PUBLIC_DOWNLOAD_URL
3. S0.1–S0.2  Live buy/refund after sandbox overlay works
4. S0.6       Polar (customer product) env → host deploy dogfood (no Studio-held secrets)

Value depth (activate one band at a time)
5. S2.4       MCP when a real ship asks
```

Earlier commerce-first order stays valid after Polar unlocks; do not block the week on Pay now.

---

## Roadmap bands (2026-09-27 snapshot)

| Band | Theme | In | Out |
|------|--------|----|-----|
| **A — Shell** | One panel per Ship concern | Targets + Integrations single nav · keep More/Run collapsed by default for Public | Per-provider sidebar trees as primary nav |
| **B — Finishable cut** | $29 hosted ops honesty | H1–H5 done · evidence clear · self-host Deploy≠serve | Fake one-click deploy · Studio OAuth success theater |
| **C — Catalog depth** | Pick lane → Open → Put → Confirm | Targets cards · Platforms Tier A–C · Payments/Email wizards | Endless logos before Polar E2E proof |
| **D — Expand lanes** | New surfaces only on demand | Steam/itch (done) · Stripe/Paddle detect · Tier D hosts · mobile listing Open | Store API upload · k8s · custody · auto-broadcast |
| **E — Agent / MCP** | Assist without lying | Contract + hostdeploy playbook | AI agent that “finishes” OAuth |
---

## Related

- [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)  
- [../CURRENT.md](../CURRENT.md) · [../handoffs/current-session.md](../handoffs/current-session.md)  
- [../../specs/backend/shipping-hub-north-star.md](../../specs/backend/shipping-hub-north-star.md)  
- [../../specs/backend/release-surface-map.md](../../specs/backend/release-surface-map.md)  
- [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
