# Get-live compose packs — peer study (FreeLLMAPI · Flowsint)

**Status:** Study **Go** · **G0 + F0–F4 + Medusa M1–M4 shipped** · **FreeLLMAPI killed until P1b+P2**  
**Updated:** 2026-10-10  
**Owner:** product / Publish General (layout queue)  
**Peers:** [freellmapi](https://github.com/tashfeenahmed/freellmapi) · [flowsint](https://github.com/reconurge/flowsint)  
**Parents:** [umami-get-live-design](./umami-get-live-design.md) · [owned-commerce-medusa-study](./owned-commerce-medusa-study-design.md) · [integrations-setup-is-not-the-product](./integrations-setup-is-not-the-product-design.md) · [ops-deploy-pivot](./ops-deploy-pivot-design.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Pack era closed — next is v0.2.5 cut (not FreeLLMAPI)
PAUSED / CANCELLED:  Third-party Integrations · Portal coach · vendor OAuth theater ·
                     FreeLLMAPI pack until P1b release + Soft-L3
CANONICAL OWNER:     On this machine / host-XOR layouts (same family as Umami)
PROOF THIS BAND:     Packs shipped; finish = release cut
```

## 1. Verdict

| Decision | Sentence |
|----------|----------|
| **Go (thin)** | Steal **setup compression** from FreeLLMAPI / Flowsint for **user-owned compose apps** — detect → ensure secrets → compose → proof URL. |
| **Kill** | Growing Desktop **Integrations** to finish Groq/OpenRouter/OSINT vendor dashboards inside Orbit. |
| **Park** | Shipping FreeLLMAPI or Flowsint as Orbit SKUs until a named outcome wall + Soft-L3 beats their official README. |

These repos are **clients that integrate many upstreams**. Orbit’s job is not to become that client — it is to make **getting that client live on the operator’s machine** shorter and less failure-prone than cloning + copying `.env` by hand (Umami delta).

## 2. What each peer actually sells

| Peer | Operator pain they absorb | Setup shape |
|------|---------------------------|-------------|
| **FreeLLMAPI** | 34 provider SDKs / keys / rate limits → **one** OpenAI `/v1` + unified key | Docker one-liner · Desktop tray · encrypted keys in SQLite · `npx … setup-<agent>` merges client config |
| **Flowsint** | Graph OSINT + enrichers that need **many** API keys | `docker compose … up` · multi-folder `.env` copies · vault for enricher keys · first register at localhost |

**Shared pattern Orbit already proved with Umami:**

```text
Docker running → secrets not defaults → compose up → URL answers → human Confirm
```

**Shared anti-pattern (CANCELLED):**

```text
Orbit finishes each upstream vendor’s account / KYC / OAuth
```

FreeLLMAPI’s Keys page and Flowsint’s enricher vault are **their** product surfaces. Orbit may Put **names**, open their Admin, and Confirm — never claim “Groq connected.”

## 3. Steal list (Orbit delta)

| Peer move | Orbit translation | Do / Don’t |
|-----------|-------------------|------------|
| One-liner / `make prod` / compose.prod | **Deploy** = compose + heartbeat (Umami U2) | Do |
| Multi `.env` copies (Flowsint ×4) | **`ensure`** writes/repairs required env files once | Do — biggest win vs their README |
| Generate `AUTH_SECRET` / vault keys | Random strong secrets into `.env` (never print values) | Do |
| Encrypted key store in-app | Their app’s vault — Orbit only scaffolds empty names / Open Admin | Don’t rebuild vault UI |
| `npx freellmapi setup-claude` | MCP `yard_setup` + pause/resume for **this** project’s cut | Do (agent spine, not Integrations) |
| Unified API key out | Proof bay: Admin URL + “copy key from *their* UI” | Soft note only |
| Catalog of 30+ enrichers/providers | Detect **one layout**, not a marketplace of vendors | Don’t |
| Failed Prisma / corrupt volume | Auto `down -v` once (Umami recovery) | Do — generalize for compose packs |
| Desktop tray that embeds the stack | Soft-no — Orbit stays sequencer; optional Later | Park |

## 4. Reusable pack contract (post-Umami)

Any new **On this machine** layout must satisfy the same contract as Umami:

| ID | Slice | Done when |
|----|-------|-----------|
| **P0** | Fixture + detect signals (compose image / marker / layout id) | Probe reports layout |
| **P1** | `orbityard <pack> ensure` — secrets / multi-env | No weak defaults left |
| **P2** | `orbityard <pack> deploy` — compose up + proof URL | Heartbeat or HTTP 200 on Admin |
| **P3** | Desktop card under **On this machine** · structure gate · Publish Confirm | Soft-L3 path shorter than official docs |
| **P4** | Corrupt-volume / crash-loop recovery (log sniff → one reset) | Soft-L3 survives teardown |

**Soft-no forever:** Integrations catalog, portal coach, “finish vendor onboarding here.”

## 5. Candidate queue (after Umami)

| Priority | Candidate | Why | Kill if |
|----------|-----------|-----|---------|
| **Next study** | **Flowsint-class** (OSINT compose + vault secrets) | Pain matches: multi-env copy + Docker + first login; Orbit `ensure` is an obvious delta | Band becomes “wire Maigret/N8n inside Studio” |
| **Killed until P1b+P2** | **FreeLLMAPI-class** | Do not start — finish-line anti-stuck ([ROADMAP](../../docs/business/ROADMAP.md)) | Band becomes “Orbit is an LLM gateway” |
| **Parked** | **Medusa** | Already studied — activate M1 when handoff says so | Stripe Connect theater |
| **Out** | Polar / Paddle / Stripe account setup | Failed Integrations thesis | — |

Flowsint is the better **next** Get-live pack to study for implementation because official Windows setup still asks for **four** `.env` copies before `compose up` — exactly where Umami-style `ensure` wins Soft-L3.

FreeLLMAPI already compresses install hard (`install.sh` / `.exe`); Orbit only wins if detect + Deploy + recovery beats “open their Desktop.” Park until measured.

## 6. Explicit non-goals

- Reviving Integrations / Portal UX expansion  
- Shipping freellmapi or flowsint forks as Orbit features  
- Auto-importing provider API keys from chat into their vaults  
- Competing with FreeLLMAPI’s catalog feed or Flowsint’s enricher plugins  

## 7. Next atomic

| ID | State |
|----|--------|
| **G0** | **Shipped** — `crates/orbityard/src/compose_pack.rs`; Umami deploy uses it |
| **F0** | **Study done** — [flowsint-get-live-design](./flowsint-get-live-design.md) |
| **F1** | **Shipped** — detect + `fixtures/flowsint-osint` |
| **F2** | **Shipped** — `orbityard flowsint ensure` multi-env |
| **F3** | **Shipped** — `orbityard flowsint deploy` compose.prod + App proof |
| **F4** | **Shipped** — Desktop card · structure gate · Publish Confirm |
| **Medusa M1–M4** | **Shipped** — [medusa-get-live](./medusa-get-live-design.md) |
| **FreeLLMAPI** | **Killed** until [v0.2.5 cut](./v0.2.5-release-plan.md) + Soft-L3 |
