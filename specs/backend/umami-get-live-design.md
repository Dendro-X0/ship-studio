# Umami Get-live — Orbit delta (option A)

**Status:** **U0–U4 done** (Soft-L3 film maintainer) · Deploy volume recovery shipped  
**Updated:** 2026-10-10  
**Owner:** `orbityard` detect · compose run · Desktop Get-live / Deployment  
**Parents:** [DEMO-AND-SIMPLICITY-HONESTY-2026](../../docs/business/DEMO-AND-SIMPLICITY-HONESTY-2026.md) · [outcome-sku-analytics-live-umami-2026](../../docs/business/outcome-sku-analytics-live-umami-2026.md) · [founder-minimal-spine](./founder-minimal-spine-design.md) · [selfhost.rs](../../crates/orbityard/src/selfhost.rs) (static only — **do not** overload)

## Plan alignment

```text
HANDOFF ATOMIC STEP: A — Umami/VPS Get-live delta so Studio < official docs
PAUSED / CANCELLED:  Integrations · fake one-click past Docker install ·
                     filming Orbit-as-Umami before this band · mid SKU
CANONICAL OWNER:     new analytics.umami lane (not selfhost.static · not container.push)
PROOF BEFORE DONE:   L1 detect fixture · L2 compose up + Admin URL ·
                     L3 silent ≤90s of *Orbit* path · L4 optional kit later
```

## 1. Honesty bar (why this exists)

Official path: copy compose → edit secrets → `docker compose up` → change admin password → add site → paste script.  
**Orbit earns its place only if** the operator path is shorter / less failure-prone:

| Official friction | Orbit delta |
|-------------------|-------------|
| Insecure default secrets | **Scaffold** generates `APP_SECRET` + DB password into `.env` (never commit) |
| “Which compose?” folklore | **Detect** Umami layout or **Init** from fixture template |
| Multi-step mental load | **One Deploy** = `compose up -d` + wait `/api/heartbeat` + print Admin URL |
| Lost in menus | Get-live spine: Open → (Docker gate) → Deploy → Open Admin → Confirm password changed |

**Still human (not fake one-click):** install Docker if missing · Confirm default password changed · DNS/TLS on a real VPS (v1 = local Docker URL; TLS = Later or runbook).

If after U3 a timed run is **not** shorter than official docs → **subtract** the lane; do not film.

## 2. Non-goals

- Hosted Umami SaaS / Coolify control plane  
- Overloading `selfhost.deploy` static serve (selfhost forbids compose-only)  
- `container.deploy` registry push theater  
- Matomo/Plausible parity in v1  
- Auto-paste tracker into customer sites  

## 3. Layout & detect

Layout id: `analytics.umami` (probe flag `umami` / `analytics_umami`).

| Signal (any strong) | Notes |
|---------------------|-------|
| Compose file references `umami-software/umami` or `docker.umami.is` image | Canonical |
| `.ship/layout.json` → `"analytics.umami"` | Opt-in |
| Fixture marker `fixtures/umami-analytics` / `UMAMI_ORBIT=1` in `.env.example` | Tests |

**Do not** treat bare `docker-compose.yml` without Umami image as this layout (container lane stays Advanced).

## 4. Operator spine (General)

| # | Step | Studio | Human |
|---|------|--------|-------|
| 1 | Open project | Detect `analytics.umami` or offer **Init Umami pack** | Bind folder |
| 2 | Doctor | Docker on PATH + daemon ping | Install Docker if loud gate |
| 3 | Ensure secrets | Write `.env` if missing (random secrets) | — |
| 4 | Deploy | `docker compose up -d` · stream · wait heartbeat | — |
| 5 | Open Admin | Open `http://127.0.0.1:PORT` (or printed URL) | Login · **Confirm** password changed |
| 6 | (Later) Tracker | Show website-id + script snippet | Paste on their site |

Done v1 = Admin URL healthy + Confirm password gate done.  
Pageview Soft-L3 = L3 demo / Later slice (script paste stays human).

## 5. Slice queue

| ID | Slice | Done when |
|----|-------|-----------|
| **U0** | Fixture `fixtures/umami-analytics` + detect + unit test | `orbityard` probe reports umami — **done** |
| **U1** | `orbityard umami ensure`: scaffold `.env` secrets if absent | No plaintext defaults in generated `.env` — **done** |
| **U2** | `orbityard umami deploy`: compose up + heartbeat + URL in last-run | CLI **done** (+ `docker-compose` fallback · **auto `down -v` once** on Prisma P3009 / failed migrate) |
| **U3** | Desktop Umami card · Deploy → Open Admin · Publish Confirm password | **done** |
| **U4** | Silent ≤90s Soft-L3 of **Orbit** path only — after L2 dogfood | L2 Pass — [evidence](../../docs/handoffs/evidence-umami-l2-dogfood-20261009.md) · film optional |
| **Later** | TLS/Caddy recipe · remote VPS SSH · tracker assist | After U4 |

## 6. CLI / Publish contracts (sketch)

```text
orbityard umami ensure --project <dir>   # .env secrets
orbityard umami deploy --project <dir>   # compose up -d + heartbeat JSON
# Publish General inserts umami.deploy (Auto) when layout detected
# Confirm step: umami.admin_password (Human) after first healthy Admin
```

Stdout JSON (deploy): `{ "ok", "admin_url", "heartbeat_ok", "message" }` — Desktop proof bay.

## 7. Proof layers

| Layer | Check |
|-------|--------|
| L1 | Fixture detect test green |
| L2 | On maintainer machine with Docker: ensure → deploy → curl Admin/heartbeat — **Pass** ([evidence](../../docs/handoffs/evidence-umami-l2-dogfood-20261009.md)) |
| L3 | Silent ≤90s: Open fixture in Desktop → Deploy → Open Admin (no voice) |
| L4 | Optional: kit SKU uses same fixture as paid pack |

## 8. Kill / subtract

| If | Then |
|----|------|
| Docker missing dominates every run | Keep loud Install Docker only; don’t pretend |
| Path longer than official README | Subtract U2–U3; park band |
| Scope → Matomo / full VPS SSH | Later design |

## 9. Next atomic

**Closed.** Further compose packs: [get-live-compose-packs-from-peers-design](./get-live-compose-packs-from-peers-design.md).
