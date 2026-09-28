# Current session — Ship Studio

**Updated:** 2026-09-27  
**Branch:** `main`  
**Status:** **Targets catalog parity Slice 1 shipped** · S1.1 shelf commit parked

## Next Atomic Step

**S1.11** Wizard completion Confirm (or **S1.1** shelf commit when maintainer asks). Soft dogfood: Integrations Polar → Continue publishing names `listing.polar`.

## Design queue (not coding until activated)

| Item | Spec |
|------|------|
| S1.10 Integrations → Publish | [integrations-publish-handoff-design](../../specs/frontend/integrations-publish-handoff-design.md) (**shipped**) |
| S0.8 update check | [in-app-update-check-design](../../specs/frontend/in-app-update-check-design.md) (**shipped** L1) |
| Integrations nav (S1.19) | [integrations-nav-design](../../specs/frontend/integrations-nav-design.md) (**shipped**) |
| Targets catalog parity | [targets-catalog-parity-design](../../specs/frontend/targets-catalog-parity-design.md) (**Slice 1 shipped** — S2/S3 queued) |
| Launch choice board | [launch-choice-board-design](../../specs/frontend/launch-choice-board-design.md) (L1–L2 shipped — L3+ queued) |
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
| Reliability Later | Non-Windows · **CDP** · vault — separate activation (no Create Token form-fill) |

## Last closed

| Band | Link |
|------|------|
| **S1.10 Integrations → Publish** | [design](../../specs/frontend/integrations-publish-handoff-design.md) — Continue publishing → listing focus |
| **S0.8 in-app update check** | [design](../../specs/frontend/in-app-update-check-design.md) — notice release · Open download |
| **Integrations nav S1.19** | [design](../../specs/frontend/integrations-nav-design.md) — single Ship Integrations item |
| **Targets catalog parity S1** | [design](../../specs/frontend/targets-catalog-parity-design.md) — Targets chrome · catalog cards · Ship nav |
| **Self-host Deploy finish-fast** | [design](../../specs/backend/selfhost-deploy-finish-fast-design.md) — Deploy = one-shot; Open live = serve |
| **Self-host serve UI freeze** | [investigation](../../specs/backend/selfhost-serve-ui-freeze-investigation.md) · `#[tauri::command(async)]` on `run_shipctl` |
| **S1.1 shelf cutover (agent)** | `/demo` → `v0.2.1` T1–T5; SCRIPT honesty vs H1–H5 Deploy; Harbor reset + scopes L1 |
| **Hosted deploy ops H1–H5** | [hosted-deploy-ops-value-bar-design](../../specs/backend/hosted-deploy-ops-value-bar-design.md) — CLI Deploy · Results · Troubleshoot · MCP `ship_hostdeploy` |
| **Hosting OAuth web entry** | [hosting-oauth-web-entry-design](../../specs/backend/hosting-oauth-web-entry-design.md) |
| **Self-host ≠ hosted Deploy** | [selfhost-vs-hosted-deploy-probe-investigation](../../specs/backend/selfhost-vs-hosted-deploy-probe-investigation.md) |
| **Launch choice board L1–L2** | Desktop Bands A/B/C + shipctl `lane`/`optional`/`suggested` |
| **Self-host S1–S5** | Catalog · stream · health · Publish Auto · detect |
| **Overhaul O5** | [evidence-harbor-client-honesty](./evidence-harbor-client-honesty.md) |
| **Overhaul O4** | [5350637](https://github.com/Dendro-X0/ship-studio/commit/5350637) |
| **Overhaul O3** | [7470dfd](https://github.com/Dendro-X0/ship-studio/commit/7470dfd) |
| **Overhaul O0–O2** | [b383a99](https://github.com/Dendro-X0/ship-studio/commit/b383a99) |

## Boot allowlist

1. This file  
2. [SCRIPT](../assets/demo/v0.2.1/SCRIPT.md) (T6/T7)  
3. `fixtures/harbor` + `scripts/harbor-reset`  
4. [demo-subject-design](../../specs/frontend/demo-subject-design.md)  
5. [hosted-deploy-ops-value-bar-design](../../specs/backend/hosted-deploy-ops-value-bar-design.md) (closed)  
