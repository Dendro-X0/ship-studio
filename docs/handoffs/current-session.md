# Current session — Ship Studio

**Updated:** 2026-09-27  
**Branch:** `main`  
**Status:** **S1.1** GIF recording — T1/T3 in hand; next T6 Sign Open + T7 Deploy Open · Self-host S1–S5 shipped

## Next Atomic Step

**S1.1 — Record `06-sign-open.gif` + `07-deploy-open.gif`** (operator). Navigation only: Sign → one official portal; Deployment → one host Open dashboard. No paid signing, no live deploy. Compress prior captures → `01-bind` / `03-confirm-next`. Shot lists: [SCRIPT](../assets/demo/v0.2.1/SCRIPT.md).

## Design queue (not coding until activated)

| Item | Spec |
|------|------|
| Launch choice board | [launch-choice-board-design](../../specs/frontend/launch-choice-board-design.md) |
| Deployment nav | [deployment-nav-design](../../specs/frontend/deployment-nav-design.md) (shipped) |
| Panel redirects | [launch-panel-redirects-design](../../specs/frontend/launch-panel-redirects-design.md) (shipped) |
| Studio self-host | [studio-selfhost-guide-design](../../specs/frontend/studio-selfhost-guide-design.md) (Slices 1–5 shipped) |

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| **Vendor handoff coach** | **CANCELLED** — [design](../../specs/backend/vendor-handoff-coach-design.md) |
| Polar paid checkout E2E | Deferred |
| aperio Advanced L4 | Parked |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |
| Drive-by Desktop reliability | Work only via design slices |
| Reliability Later | Non-Windows · CDP · vault — separate activation |

## Last closed

| Band | Link |
|------|------|
| **Self-host detect (S5)** | `plan_eligible` · `.ship/selfhost.json` root · Dockerfile-only skips Self-host |
| **Self-host Publish Auto (S4)** | `selfhost.deploy` on General+Advanced + Launch; Continue/Verify runs checks |
| **Self-host health (S3)** | Artifact + local GET 200 → last-run `selfhost.check`; no Confirm |
| **Self-host stream (S2)** | `shipctl selfhost` + Deployment **Deploy** streams; Harbor `apps/website` |
| **Self-host catalog (S1)** | Deployment **Self-host** card; Orbit gated on `orbit_configured` |
| **Sign catalog grid** | Official signing cards restored on Sign (Apple · Microsoft · Play · GitHub) — same layout as Deployment / Integrations |
| **Overhaul O5** | [evidence-harbor-client-honesty](./evidence-harbor-client-honesty.md) |
| **Overhaul O4** | [5350637](https://github.com/Dendro-X0/ship-studio/commit/5350637) |
| **Overhaul O3** | [7470dfd](https://github.com/Dendro-X0/ship-studio/commit/7470dfd) |
| **Overhaul O0–O2** | [b383a99](https://github.com/Dendro-X0/ship-studio/commit/b383a99) |

## Boot allowlist

1. This file  
2. [SCRIPT](../assets/demo/v0.2.1/SCRIPT.md)  
3. `fixtures/harbor` + `scripts/harbor-reset`  
4. [demo-subject-design](../../specs/frontend/demo-subject-design.md)  
