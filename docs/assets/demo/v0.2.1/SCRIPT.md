# Feature demo — v0.2.1 (Part 1)

**Status:** Recording plan for S1.1 live GIFs  
**Build:** App already installed ([v0.2.3](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.3)+) — **do not record setup/install**  
**Watermark:** `local app · recorded`  
**Replace:** stylized `docs/assets/demo/v0.1.0/` → live clips under `docs/assets/demo/v0.2.1/` (same file names when possible)

## Demo stance

Prove **useful mid-flight shipping**, not “how to install software.” Everyone can install an app; the GIF shelf must show bind → detect → Continue/Open/Confirm in under a minute of watching.

**Honesty for Sign / Deploy clips:** Studio **opens the official site** (or Login CLI). Do **not** record paid notarization, store upload, live `wrangler deploy`, or every host. One host + one official portal each is enough.

## Project choice

| Option | Use when | Verdict for Part 1 |
|--------|----------|--------------------|
| **`fixtures/harbor`** | Lightweight Desktop + Docs (+ Signet) — clean Targets for GIFs | **Primary — always** |
| `fixtures/advanced-dogfood` | Advanced multipath dogfood (mobile · CI · container · steam) | Engineering only — not Part 1 shelf |
| **Ship Studio itself** | Engineering dogfood on the real monorepo | **Never for public GIFs** — already signed/deployed |
| **aperio** | Extreme Targets / Advanced Public mid-flight | **Avoid for public GIFs** |

**Rule:** Bind Harbor (`fixtures/harbor`) from the installed Desktop. Run `scripts/harbor-reset` (or wipe `.ship/`) before recording so Scopes / Configure / Sign·Deploy start mid-flight. Prefer **Advanced + Local** for bind + publish check. For **Deployment Open** only, switch intent to **Public** so the Deployment panel is meaningful — still do not run a live deploy.

**Client honesty (overhaul O5):** Prefer Local for bind/confirm. Deployment clip may briefly use Public + **Open dashboard** on one host (Cloudflare or Orbit). Never Docs-as-path. Proof: [evidence-harbor-client-honesty](../../handoffs/evidence-harbor-client-honesty.md).

Design: [demo-subject-design](../../../specs/frontend/demo-subject-design.md).

## What Part 1 proves (one sentence)

Studio keeps **mid-flight order** across surfaces: you Open vendor doors, Confirm gates, and advance — a chatbot checklist cannot hold that state.

## Features to showcase (in)

| # | Feature | Why it sells |
|---|---------|--------------|
| 1 | **Bind project** | Local-first; one window = one repo |
| 2 | **Targets sidebar** | Multi-surface detection (Web / API / Desktop cues in fixture) |
| 3 | **Publish portal** | Step N/M spine — doctor → scopes → env → sign → … |
| 4 | **Open / Run** | Browser/terminal for the *human* gate (Studio does not OAuth) |
| 5 | **Continue** (+ Confirm when Human) | Fast path burns Auto gates; Open→Confirm only for honesty |
| 6 | **Output Preview** | Local artifact / plan JSON before the next Confirm |
| 7 | **Back** (← Back / Back to Publish) | Mid-flight return after visiting Sign / Deployment |
| 8 | **Sign → Open portal** | Navigate Apple / Microsoft / Partner / GitHub — no paid certs |
| 9 | **Deployment → Open dashboard** | One host card → vendor UI — no live deploy |

## Out of Part 1 (do not record)

- Live Polar Pay / KYC charge  
- Live `docker push` / npm publish / store upload / notarization / Authenticode purchase  
- Deploying **every** host (pick **one**: Cloudflare **or** Orbit)  
- In-app update check (S0.8)  
- Full aperio 19-step Public grind  
- Website checkout / license email (site demo, not Desktop Part 1)

## Capture resolution (GIF shelf)

`/demo` displays clips at **720px** wide (`demo.astro` + existing v0.1.0 assets are **720×420**). Export to that; do not ship 1080p GIFs.

| Stage | Size | Notes |
|--------|------|--------|
| **Export (canonical)** | **720×420** | Same as v0.1.0 shelf · ~12:7 · keeps `/demo` weight sane |
| **Capture (Desktop)** | **960×560** window (or crop) | Readable sidebar + Publish row; then **downscale → 720×420** |
| Avoid | ≥1280×720 as final GIF | File size explodes; shelf only shows ~720 CSS px |
| Target weight | **≤1.5–3 MB** per clip | 8–12 fps · short palette — raw 60–130 MB captures must be compressed |

### How to set it

1. Resize the Ship Studio window to about **960×560** (titlebar included). Default app size is larger — shrink before recording.  
2. Record **only the app window** (no desktop wallpaper). Optional: cut **before** the browser tab fills the frame, or crop to Studio only.  
3. Export / convert to GIF at **720×420**, **8–12 fps**, **8–15 s** per clip.  
4. Rename once: `select_project.gif` / `check_publish.gif` — avoid `.gif.gif`.

Crop rule if the window must stay larger: keep **sidebar + Publish/Sign/Deployment chrome** in frame; drop the bottom Output dock if it steals height.

## Recording tasks (ordered)

### Prep (off-camera)

1. App already running (installer or portable) — **no install / first-run wizard in any GIF**.  
2. Reset Harbor: `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1` (or delete `fixtures/harbor/.ship/`).  
3. Window ~**960×560**; Output dock **hidden** (statusbar Preview only).  
4. Hide personal paths if possible (`…\fixtures\harbor`).  
5. Capture: silent GIF or short MP4→GIF; no voiceover required for `/demo`.

### Beats → files (keep names stable for `/demo`)

| Task | Clip file | Your recording (map) | Action on screen (~8–12s) | On-page caption |
|------|-----------|----------------------|---------------------------|-----------------|
| **T1** | `01-bind.gif` | `select_project…` | Empty Dashboard → **Open** → `harbor` → Targets | Pick a folder. Studio detects the ship layout — no cloud account. |
| **T2** | `02-open.gif` | *(optional auth)* | Open Deployment / Login CLI **or** skip if T7 covers Open | Open the right surface. Studio does not OAuth for you. |
| **T3** | `03-confirm-next.gif` | `check_&_publish…` | Publish check → Confirm / Continue (no official Sign, no live deploy) | You attest human gates; Continue burns the rest. |
| T4 | `04-output-preview.gif` | | Statusbar **Preview** | Inspect local output when you need it. |
| T5 | `05-midflight.gif` | | Mid-flight → Sign or Deployment → **← Back** | Mid-flight state survives. |
| **T6** | `06-sign-open.gif` | **record next** | Ship → **Sign** → Open one official portal (e.g. Apple or Partner Center) → cut before paying | Studio opens the store / cert site — you finish there. |
| **T7** | `07-deploy-open.gif` | **record next** | Ship → **Deployment** → pick **one** host → **Open dashboard** → cut (no deploy) | One host door. Studio never deploys for you. |

### T1 shot list (done / re-export)

```text
0–1s   Empty / “Choose a project” Dashboard
1–3s   Click Open
3–6s   Folder picker → harbor
6–10s  Dashboard: name · Targets Desktop–Tauri / Website–Static
```

### T3 shot list (done / re-export — Local, no official Sign / no live deploy)

```text
0–2s   Publish mid-flight (or Launch → Publish)
2–8s   Pre-publish check / Confirm · Continue through Auto gates
8–12s  Hold on clear “next human gate” — stop before Signet release / Orbit deploy run
```

### T6 — Official signing Open (navigation only)

```text
Prep    Harbor bound · Advanced · Local · Sign panel ready (Check status optional)
0–2s    Sidebar → Sign (or Launch → Open Sign)
2–5s    Official path visible (Apple / Microsoft / GitHub — pick ONE)
5–9s    Click Open / Open dashboard — browser may flash; cut as soon as URL/title is clear
9–11s   Optional: ← Back to Studio (shows return) — then cut
```

**Do not show:** certificate purchase, notarization wait, App Store Connect submit, Partner Center paywall, live `signet release` upload.

### T7 — Deployment Open (one provider only)

```text
Prep    Harbor bound · switch intent to Public for this clip only · Deployment panel
0–2s    Sidebar → Deployment
2–5s    Select ONE card (prefer Cloudflare or Orbit — whatever is Suggested)
5–9s    Open dashboard (or Login CLI if you want auth assist — not both hosts)
9–11s   ← Back — cut. Never click Deploy / ship / wrangler deploy in-frame.
```

**Do not show:** multi-host tour, live deploy logs, production URL claim, Docs/Learn more as the hero click.

### Drop to site

1. Compress / rename → `docs/assets/demo/v0.2.1/01-bind.gif` … `07-deploy-open.gif`.  
2. Mirror under `apps/website/public/demo/v0.2.1/`.  
3. Point `/demo` at `v0.2.1` when shelf is ready.  
4. Update captions here if wording changes.

## Presenter line (say or subtitle)

> Ship Studio sequences the final mile — sign, release, deploy — and remembers where you are. Vendors and you still finish the irreversible steps.

## Success criteria

- [ ] **No install / setup** footage in any clip  
- [ ] T1 alone proves usefulness: folder → detected layout + clear next action  
- [ ] T3 = publish check / Confirm — **no** paid signing, **no** live provider deploy  
- [ ] T6 / T7 = **Open official site** only (one portal each)  
- [ ] Final GIF size **720×420**, preferably **≤3 MB** (re-export if still 60–130 MB)  
- [ ] No console flash during Verify / page switches  
- [ ] No claim of one-click deploy / auto OAuth / live publish  
- [ ] Fixture path only — no aperio secrets on screen  
