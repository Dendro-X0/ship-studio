# Studio self-host — local auto lane (design)

**Status:** Slices 1–5 shipped — band complete (Later: richer probes / MCP)  
**Updated:** 2026-09-27  
**Parents:** [platforms-catalog-design](./platforms-catalog-design.md) · [container-final-mile-design](../backend/container-final-mile-design.md) · [verify-status-layers-design](../backend/verify-status-layers-design.md) · [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md)  
**Trigger:** Orbit Hosting card is not a SaaS host; operators want **self-host** as the lane Studio can actually own. Self-host is the easiest **local** feature to automate and the most controllable by Studio.

```text
GOAL:  Self-host = Studio-controlled local deploy lane: stream the run, finish with
       non-step checks (disk / local CLI verify) — no Confirm→Next spine for the cut itself.
NOT:   Multi-tenant Studio SaaS · vendor OAuth deploy · docker push from bridge ·
       pretending self-host ≥ Docker/container maturity today · Dendro datacenter
```

## Problem

1. **Orbit misread** — listed like Cloudflare; operators without Orbit config (or without wanting that stack) bounce.
2. **Wrong automation bar** — early draft treated self-host like another Open-dashboard guide. Vendor hosts *must* stay human-gated. Self-host should not.
3. **Docker is the cousin** — Advanced already **Runs** `container.build` locally and streams via terminal; push stays Confirm. Self-host should feel *similar* (local, Studio-driven, streamed) but is **not yet as good** as the container lane (no shared image model, weaker host story, earlier maturity).

## North star — two deploy classes

| Class | Examples | Studio control | Finish rule |
|-------|----------|----------------|-------------|
| **Vendor host** | CF · Vercel · Netlify · Pages · Fly · … | Low — portal / Login CLI / Open | Human **Confirm** after vendor UI |
| **Local auto** | **Self-host** · `container.build` (partial) | High — Studio owns the process | **Non-step checks** (verify layers) auto-complete; no Confirm for the local cut |

Self-host is the **flagship local-auto** lane. Containers are the nearest shipped analog: good local build Run, but registry push still Confirm — so container deploy is *better tooling today*, while self-host aims for *more end-to-end auto* on the local path once sliced.

## Mental model

| Layer | Authority | Self-host behavior |
|-------|-----------|-------------------|
| Local process (serve / static preview / compose-up / studio site build) | Studio + operator machine | **Stream** stdout/stderr in Desktop terminal; operator watches progress |
| Done? | Disk + local CLI ([verify-status-layers](../backend/verify-status-layers-design.md)) | Probe URL / port / artifact / last-run — **auto mark done** when checks pass |
| Public SaaS cutover | Operator + vendor | Out of this lane — use Hosting cards + Confirm |
| Orbit CLI | Optional when `orbit_configured` | Separate card; never “your datacenter” |

**Invariant:** Bridge still never calls vendor HTTPS with secrets and never `docker push`. Self-host only automates what runs **on the bound machine**.

## Contrast — Docker / container

| | Container (today) | Self-host (target) |
|--|-------------------|--------------------|
| Detect | Dockerfile / Compose | Self-host signals (Studio monorepo, static site, compose-up intent, `.ship/selfhost` later) |
| Local build/run | `docker build` / `compose build` **Run** | Streamed self-host deploy/serve command |
| Finish local | Operator **Confirm** after build | **Non-step check** (process up · health URL · artifact) |
| Push / registry | Confirm-only docs | N/A (local) or later optional |
| Maturity | Stronger shipped slice | Weaker until slices land — copy must not claim parity |

Product copy: “Like a local container run — Studio streams it and checks it — not a cloud host. Container layouts stay on the Docker lane when Dockerfile wins detect.”

## Target UX

### Placement

| Surface | Behavior |
|---------|----------|
| **Deployment** catalog | **Self-host** card (not Orbit-as-SaaS). Orbit only if `orbit_configured`. |
| Detail panel | **Deploy** primary (not Open dashboard). Live stream + check status. |
| Publish / Launch | Optional `selfhost.deploy` as **Auto** step (Continue/script) — omitted from Human-gate Confirm list when checks own completion. |
| CmdK | “Self-host deploy” → Deployment + start or focus stream. |

### Streaming deploy

1. Operator hits **Deploy** (or Publish Continue on `selfhost.deploy`).
2. Desktop/terminal attaches to the local command (reuse Ritual/Tools stream path).
3. UI shows phase: starting → streaming → checking → done / failed.
4. No Confirm toast for success — status flips when non-step checks pass.

### Non-step checks (completion)

Ordered probes (stop at first hard fail):

| Check | Layer | Example |
|-------|-------|---------|
| Exit code | local CLI | Process ended 0 for one-shot builds |
| Listener / health | local CLI | `http://127.0.0.1:<port>/` or configured health path |
| Artifact | disk | `dist/` / installer staging path exists |
| Last-run | disk | `.ship/last-run.json` ok + kind `selfhost` |

Publish progress: mark `selfhost.deploy` **Done** from verify — same honesty as other local_cli layers; never claim vendor Live check.

### Orbit policy

| Detect | Catalog |
|--------|---------|
| No Orbit | Hide Orbit; show **Self-host** |
| Orbit configured | Orbit = “Local Orbit CLI → your CF/Vercel/Netlify”; Self-host remains for Studio-owned local auto |

## Non-goals

- Dendro-operated Studio cloud / shared CDN  
- Auto `docker push` / registry login  
- Replacing vendor Hosting cards  
- Claiming self-host ≥ Docker UX in v1  
- Changing Harbor GIF T7 host picks (CF/Vercel/Pages)

## Slices

| Slice | Work | Proof |
|-------|------|-------|
| **0 — Spec** | This file + handoff queue | Review |
| **1 — Catalog honesty** | Self-host card; gate Orbit; Docker-cousin blurb | **Done** (L1 `tsc`) |
| **2 — Stream Deploy** | Wire Deploy → local command + terminal stream | **Done** — `orbityard selfhost` + Deployment Deploy; L1 tests · Harbor static |
| **3 — Non-step checks** | Health/artifact/last-run → auto Done | **Done** — bind + GET 200 · `selfhost.check` in last-run |
| **4 — Publish Auto step** | `selfhost.deploy` Continue path when signals present | **Done** — General+Advanced · Launch parity · verify runs selfhost |
| **5 — Detect matrix** | Studio monorepo · static preview · optional compose-up without stealing Dockerfile layouts | **Done** — `plan_eligible` · opt-in root · Dockerfile-only skips |
| **Later** | Richer probes · MCP watch · parity polish vs container | Separate activation |

## Owners (when activated)

| Concern | Owner |
|---------|--------|
| Catalog / blurb | `apps/desktop/src/platforms-data.ts` |
| Stream UI | Desktop Ritual/terminal + Deployment wizard |
| Run + verify | `orbityard` deploy/selfhost + verify layers |
| Plan step | `publish.rs` / `launch.rs` Auto step |

## Proof (band done)

| Layer | Check |
|-------|--------|
| L1 | `tsc` · `cargo test -p orbityard` selfhost units |
| L2 | Harbor Deployment → Self-host **Deploy** streams; Done without Confirm |
| L2 | Fail health → stays not-done; no false Live |
| L2 | Dockerfile project still prefers container lane, not Self-host stealing |

## Activation

Slices 1–5 shipped (Self-host local-auto band complete). Later = richer probes / MCP watch — separate activation. S1.1 GIF recording remains the session atomic step.
