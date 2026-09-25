# Ship Studio — Scope of Service

**Status:** Active (product definition)  
**Updated:** 2026-09-25  
**Canonical with:** [PRODUCT.md](./PRODUCT.md) · [PLATFORMS-AND-PORTAL.md](./PLATFORMS-AND-PORTAL.md) · [shipping-hub-north-star.md](../../specs/backend/shipping-hub-north-star.md) · [OPERATOR-NEXT.md](./OPERATOR-NEXT.md) · [verify-status-layers-design](../../specs/backend/verify-status-layers-design.md)

## One sentence

Ship Studio is a **local portal and guide** for release work: it sequences the **final mile** (**sign → release → deploy** and adjacent lanes) across many project kinds, runs **scripts for highly automatable steps**, and uses **semi-automated wizards** when official channels require the human — without replacing vendor platforms.

## Value thesis (why pay vs SaaS starter kits)

Indie and multi-repo operators (dozens of OSS repos, a few commercialized) lose time on **order and surfaces**, not on “another template.” Ship Studio earns its price when the same tool makes **OSS cuts, marketplace listings, host deploys, and commerce gates** faster to complete and harder to miss — practical utility over generic starter kits.

---

## Who we serve

| Primary | Secondary (not yet) |
|---------|---------------------|
| Solo / small teams with **many repos** (OSS + a few paid products) who need one local hub for diverse release workflows | Large orgs needing multi-tenant cloud control planes |
| Operators who ship **multi-surface** products (Web/API + desktop Signet + optional stores/commerce) | Teams that want Ship Studio to *replace* Cloudflare / Polar / store consoles |
| Operators who already use (or will install) **Signet**, **Orbit**, **gh**, provider CLIs | People who only need a single `wrangler deploy` |
| People who lose time on **“what next / which dashboard / which script”** | People seeking full auto-publish without human attestation |

---

## In scope (the service)

### 1. Detect

Probe a **local project directory** for shipping signals (Wrangler/Vercel/Netlify, Tauri/signet.toml, Dockerfile, stores, markets, CI release workflows, legal/trust files, suite.json, etc.).

### 2. Plan

Build an **Adaptive Publish plan** (General = short path; Advanced = full OAuth / listing / submit / CI / container / marketing / suite):

- Ordered steps with honest detail  
- Related `desktop_view` to the right detail panel  
- Safe local **Run** / **Continue** scripts where a CLI exists (Signet, Orbit, `docker build`, `gh` list, npm/cargo `--dry-run`, …)  
- **Open** URLs + Confirm wizards when the human must act on an official channel  

### 3. Sequence

Drive the minute spine on **CLI · TUI · Desktop · MCP**:

**Continue** (Auto / scriptable) · **Open / Run → (human + vendor) → Confirm → Next** (official-channel gates)

Progress lives in project `.ship/publish.json` (no secret values). Desktop shows **done · required · later**. Optional **Watch** polls local Verify — still never finishes OAuth/store/DNS for you.

### 4. Orient

- **Doctor** — tools required *for this layout*  
- **Pulse** — Dashboard Now / Continue mid-publish with step-specific cut hints  
- **Assist** — checklist overview (not a second wizard)  
- **Portal / Env / Secrets** — entry URLs + paste-put assist; optional encrypted `.km` vault export  

### 5. Honesty

- Offline bridge: Studio does **not** call vendor HTTPS with secrets  
- Never store secret values in `.ship/` plaintext  
- Never claim verified publisher / SmartScreen silence unless true  
- Never auto-`docker push`, live `npm|cargo publish`, or `gh release create`  
- Desktop-only ships: Signet release is the deploy (no fake Orbit desktop host)  
- **Status layers** ([verify-status-layers-design](../../specs/backend/verify-status-layers-design.md)): **disk** · **local CLI** · **official CLI probe** (operator-gated Verify/Watch) · **human attest** — provider remains authority for irreversible done  

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
│ Status: disk · local CLI · official CLI probe         │
│ Local CLI Runs (safe / dry-run / read-only preferred) │
└───────────────────────────┬───────────────────────────┘
                            │ Open / Run handoff
┌───────────────────────────▼───────────────────────────┐
│ Human + vendor platforms (authority for irreversible) │
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
