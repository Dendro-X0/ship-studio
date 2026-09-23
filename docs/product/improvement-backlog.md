# Ship Studio — improvement backlog

**Status:** Living idea list — **not** an activate queue  
**Updated:** 2026-09-22  
**Cadence:** Pick from here when energy allows; hub Adaptive stays idle unless a real ship needs a lane  
**Canon:** [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · [PRODUCT.md](./PRODUCT.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md) · [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
**Strategy sim:** `strategy-research-lab/strategies/sims/S-SHIP-STUDIO-hub.md`

```text
Value bar:  Multi-surface final-mile order + honest Verify
Not:        Hype · chatbot-substitutable checklists · fake auto-publish
Price:      $29 one-time (Polar) — paid delta must stay real
```

Every candidate should still answer: Does it reduce missed gates for multi-surface ships? Does it stay inside scope (no vendor replacement, no dangerous live publish)? Would a stranger feel this was worth $29 — or only a pasted README?

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
| S0.3 | **Public download path** | Done — [v0.1.0 Windows zip](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.1.0) + SHA256 |
| S0.4 | **Paid delta on `/pricing`** | Done (`28f696e`) |
| S0.5 | **Brand disambiguation** | Done (`28f696e`) |
| S0.6 | **Polar env wizard → deploy** | Checklist already in Desktop Integrations; one dogfood that `PUBLIC_POLAR_*` lands on website deploy without Studio writing secrets |

---

## P1 — Perception of real value (anti-hype)

| ID | Idea | Notes |
|----|------|-------|
| S1.1 | **Demo shelf = multi-target tedium** | GIF/script shows Desktop + Docs/website + Polar Confirm chain — proof a chatbot can’t hold mid-flight state |
| S1.2 | **Dashboard honesty polish** | Surface Signet ok / Orbit missing, dirty git, step N/M without fake “all shipped” |
| S1.3 | **Never-say block on site + README** | No “one-click deploy” · no “we silence SmartScreen” · no “replaces Cloudflare/Polar” · Studio does not write secrets |
| S1.4 | **One intent page** | “Multi-surface final-mile vs CI-only / checklist” — SEO without belonging theater |
| S1.5 | **OPERATOR-NEXT as product feature** | Market human gates as honesty, not unfinished bugs |
| S1.6 | **Cafe / FAQ stub** | SmartScreen? vs other Ship Studio? Why $29? Signet required? |
| S1.7 | **Offline badge meaning** | Short copy: bridge offline; Open/login/put are operator-initiated |

---

## P1 — Hub UX (Integrations / Publish)

| ID | Idea | Notes |
|----|------|-------|
| S1.10 | **Integrations → Publish handoff** | From Polar wizard, clear “Continue publishing” with next plan step — less sidebar archaeology |
| S1.11 | **Wizard completion Confirm** | After Open dashboard + paste env, explicit Confirm so `.ship/publish.json` advances |
| S1.12 | **Provider “done” criteria** | Per Polar/Stripe/…: what Verify checks (URL present vs webhook live) — stay honest |
| S1.13 | **Resend / email wizard parity** | Same Open → checklist → Confirm pattern as Polar |
| S1.14 | **Local vs Public intent copy** | Make Local/Public toggle consequences one sentence each on Dashboard |
| S1.15 | **Dirty-tree Publish cue** | Dirty-on-main warnings: optional soft gate before release steps |
| S1.16 | **Modularization M2+** | Continue extract sidebar / integrations-ui / output-preview / cmdk when continuing desktop work |

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

## Suggested order (v0.1.0 release week — active)

```text
1. S0.4–S0.5  Paid delta + brand disambiguation   ← now
2. S0.3       GitHub Release + SHA256 + download URL
3. S1.1       Live silent demo GIFs (replace stylized set)
4. Deploy site · Polar Success/Return URLs
5. S0.1–S0.2  Live buy/refund only when Polar payment_ready (not a release blocker)
```

Earlier default order (commerce-first) stays valid after Polar unlocks; do not block the week on Pay now.
---

## Related

- [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)  
- [../CURRENT.md](../CURRENT.md) · [../handoffs/current-session.md](../handoffs/current-session.md)  
- [../../specs/backend/shipping-hub-north-star.md](../../specs/backend/shipping-hub-north-star.md)  
- [../../specs/backend/release-surface-map.md](../../specs/backend/release-surface-map.md)  
- [../../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
