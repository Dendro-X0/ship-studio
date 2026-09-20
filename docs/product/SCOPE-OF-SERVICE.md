# Ship Studio — Scope of Service

**Status:** Active (product definition)  
**Updated:** 2026-09-19  
**Canonical with:** [PRODUCT.md](./PRODUCT.md) · [shipping-hub-north-star.md](../../specs/backend/shipping-hub-north-star.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)

## One sentence

Ship Studio is a **local, offline-first shipping hub** that sequences the **final mile** of a software product — **sign → release → deploy** — so operators rarely miss a gate, while **vendor platforms and the human** still perform OAuth, review, DNS, and secret creation.

---

## Who we serve

| Primary | Secondary (not yet) |
|---------|---------------------|
| Solo / small teams shipping **multi-surface** products (Web/API + desktop Signet + optional stores/commerce) | Large orgs needing multi-tenant cloud control planes |
| Operators who already use (or will install) **Signet**, **Orbit**, **gh**, provider CLIs | Teams that want Ship Studio to *replace* those tools |
| People who lose time on **order and surfaces** (“what next / which dashboard”) | People who only need a single `wrangler deploy` |

---

## In scope (the service)

### 1. Detect

Probe a **local project directory** for shipping signals (Wrangler/Vercel/Netlify, Tauri/signet.toml, Dockerfile, stores, markets, CI release workflows, legal/trust files, suite.json, etc.).

### 2. Plan

Build an **Adaptive Publish plan** (General = short path; Advanced = full OAuth / listing / submit / CI / container / marketing / suite):

- Ordered steps with honest detail  
- Related `desktop_view` to the right detail panel  
- Safe local **Run** commands where a CLI exists (Signet, Orbit, `docker build`, `gh` list, npm/cargo `--dry-run`, …)  
- **Open** URLs for vendor UIs when the human must act  

### 3. Sequence

Drive the minute spine on **CLI · TUI · Desktop · MCP**:

**Open / Run → (human + vendor) → Verify / Confirm → Next**

Progress lives in project `.ship/publish.json` (no secret values). Optional **Watch** (`shipctl publish watch` / Desktop toggle) polls local Verify and prompts when a step is ready — still never finishes OAuth/store/DNS for you.

### 4. Orient

- **Doctor** — tools required *for this layout*  
- **Pulse** — Dashboard Now / Continue mid-publish with step-specific cut hints  
- **Assist** — checklist overview (not a second wizard)  
- **Portal / Env / Secrets** — entry URLs + paste-put assist; optional encrypted `.km` vault export  

### 5. Honesty

- Offline bridge: Studio does **not** call vendor HTTPS by itself  
- Never store secret values in `.ship/` plaintext  
- Never claim verified publisher / SmartScreen silence unless true  
- Never auto-`docker push`, live `npm|cargo publish`, or `gh release create`  
- Desktop-only ships: Signet release is the deploy (no fake Orbit desktop host)  

---

## Out of scope (not the service)

| Category | Examples |
|----------|----------|
| **Content / marketing** | Product docs sites, feature demos, GIF authoring, narrative copy |
| **Vendor replacement** | Replacing Cloudflare, Vercel, Apple, Play, Gumroad, Steam UIs |
| **Finishing without the human** | OAuth completion, store review, DNS cutover, certificate purchase |
| **Dangerous automation** | Registry push, live package publish, store API upload, k8s controllers |
| **Secret custody as product** | Becoming the team’s password manager (vault export is optional backup only) |
| **Build-system replacement** | Rewriting Signet / Orbit / provider CLIs inside this repo |
| **Portfolio SaaS (v0)** | Multi-root cloud hub, paid unlock bands, hosted multi-tenant control plane |

Human remaining work is listed in [OPERATOR-NEXT.md](./OPERATOR-NEXT.md) — that checklist is **part of the service boundary**, not a backlog of Studio bugs.

---

## Responsibility split

```text
┌─ Ship Studio ─────────────────────────────────────────┐
│ Detect · Plan · Sequence · Orient · Honest Verify     │
│ Local CLI Runs (safe / dry-run / read-only preferred) │
└───────────────────────────┬───────────────────────────┘
                            │ Open / Run handoff
┌───────────────────────────▼───────────────────────────┐
│ Human + vendor platforms                              │
│ Tokens · OAuth · store review · DNS · live publish    │
│ docker push · npm/cargo publish · release create      │
└───────────────────────────────────────────────────────┘
```

**Studio succeeds** when the operator always knows the next gate and has a one-action path to the right CLI or dashboard.  
**Studio does not fail** when the vendor rejects a submission or DNS is wrong — that remains operator/vendor responsibility.

---

## Delivery surfaces (how the service is consumed)

| Surface | Role |
|---------|------|
| `shipctl publish` | Preferred minute wizard |
| Desktop (Tauri) | Same spine + Related panels + pulse CTA |
| `shipctl tui` | Terminal Publish (`P`) |
| MCP | Agent-accessible publish/guide/portal tools |

Detail panels (Scopes, Env, Sign, Portal, Ritual, Tools) open **from the current Publish step** — not competing start points.

---

## Success criteria (service outcomes)

1. For a bound project, Advanced/General plan matches layout (no nonsense Orbit desktop deploy, no missing CI after release).  
2. Operator can complete a cut without assembling eight nav destinations by hand.  
3. Mid-flight state survives restart (`.ship/publish.json`).  
4. Dogfood: `cargo test -p shipctl` · `scripts/dogfood-advanced-*.sh`.  
5. Real proof: publish at least one of *your* products end-to-end using Studio for sequence.

---

## Allowed future growth (still in scope)

- New Adaptive lanes **only when a real ship needs them** (release-surface map)  
- Deeper safe Runs / Verify honesty  
- Better Desktop/TUI ergonomics on the **same spine**  
- Suite URL sync and multi-product paste cues  

## Deferred / not promised

- Mobile store API upload  
- Kubernetes / Helm automation  
- Hosted multi-tenant SaaS control plane  
- Auto-finishing OAuth or store review  

---

## Positioning line (release)

> **Ship Studio** — local final-mile hub: sign → release → deploy, sequenced. Vendors and humans still do the irreversible bits.
