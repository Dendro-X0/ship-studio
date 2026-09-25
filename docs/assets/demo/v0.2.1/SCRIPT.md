# Feature demo — v0.2.1 (Part 1)

**Status:** Recording plan for S1.1 live GIFs  
**Build:** App already installed ([v0.2.3](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.3)+) — **do not record setup/install**  
**Watermark:** `local app · recorded`  
**Replace:** stylized `docs/assets/demo/v0.1.0/` → live clips under `docs/assets/demo/v0.2.1/` (same file names when possible)

## Demo stance

Prove **useful mid-flight shipping**, not “how to install software.” Everyone can install an app; the GIF shelf must show bind → detect → Continue/Open/Confirm in under a minute of watching.

## Project choice

| Option | Use when | Verdict for Part 1 |
|--------|----------|--------------------|
| **`fixtures/harbor`** | Lightweight Desktop + Docs (+ Signet) — clean Targets for GIFs | **Primary — always** |
| `fixtures/advanced-dogfood` | Advanced multipath dogfood (mobile · CI · container · steam) | Engineering only — not Part 1 shelf |
| **Ship Studio itself** | Engineering dogfood on the real monorepo | **Never for public GIFs** — already signed/deployed |
| **aperio** | Extreme Targets / Advanced Public mid-flight | **Avoid for public GIFs** |

**Rule:** Bind Harbor (`fixtures/harbor`) from the installed Desktop. Run `scripts/harbor-reset` (or wipe `.ship/`) before recording so Scopes / Configure / Sign·Deploy start mid-flight. Prefer **Advanced + Local** for the spine. Use **Public** only for a short Polar Open beat if needed.

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
| 7 | **Back to Publish** (optional 5th clip) | Mid-flight return after visiting Env / Sign — chatbot-killer |

## Out of Part 1 (do not record yet)

- Live Polar Pay / KYC charge  
- Live `docker push` / npm publish / store upload  
- In-app update check (S0.8)  
- Full aperio 19-step Public grind  
- Website checkout / license email (site demo, not Desktop Part 1)

## Capture resolution (GIF shelf)

`/demo` displays clips at **720px** wide (`demo.astro` + existing v0.1.0 assets are **720×420**). Export to that; do not ship 1080p GIFs.

| Stage | Size | Notes |
|-------|------|--------|
| **Export (canonical)** | **720×420** | Same as v0.1.0 shelf · ~12:7 · keeps `/demo` weight sane |
| **Capture (Desktop)** | **960×560** window (or crop) | Readable sidebar + Publish row; then **downscale → 720×420** |
| Avoid | ≥1280×720 as final GIF | File size explodes; shelf only shows ~720 CSS px |

### How to set it

1. Resize the Ship Studio window to about **960×560** (titlebar included). Default app size is larger — shrink before recording.  
2. Record **only the app window** (no desktop wallpaper).  
3. Export / convert to GIF at **720×420**, **8–12 fps**, **8–15 s** per clip.  
4. Target **≤1.5 MB** per file when possible (palette + fps matter more than a few extra pixels).

Crop rule if the window must stay larger: keep **sidebar + Publish header + step list** in frame; drop the bottom Output dock if it steals height — Output has its own clip (T4).

## Recording tasks (ordered)

### Prep (off-camera)

1. App already running (installer or portable) — **no install / first-run wizard in any GIF**.  
2. Reset Harbor: `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1` (or delete `fixtures/harbor/.ship/`).  
3. Window ~**960×560**; Output dock **hidden** (statusbar Preview only).  
4. Hide personal paths if possible (`…\fixtures\harbor`).  
5. Capture: silent GIF or short MP4→GIF; no voiceover required for `/demo`.

### Beats → files (keep names stable for `/demo`)

| Task | Clip file | Action on screen (~8–12s) | On-page caption |
|------|-----------|---------------------------|-----------------|
| **T1** | `01-bind.gif` | App already open on empty Dashboard → **Open** → pick `harbor` → Dashboard **Now** + Targets (Desktop · Docs) visible | Pick a folder. Studio detects the ship layout — no cloud account. |
| T2 | `02-open.gif` | Publish → progress bands → **Needs Open** / Open into a detail panel → return | Open the right surface. Studio does not OAuth for you. |
| T3 | `03-confirm-next.gif` | **Confirm** → **Continue** (or Next) through 1–2 gates | You attest human gates; Continue burns the rest. |
| T4 | `04-output-preview.gif` | Statusbar **Preview** on plan / doctor JSON | Inspect local output when you need it. |
| T5 | `05-midflight.gif` | Mid-flight → Env or Sign → **Back to Publish** still on step N/M | Mid-flight state survives. A pasted checklist cannot. |

### T1 shot list (record this first)

```text
0–1s   Empty / “Choose a project” Dashboard (no install chrome)
1–3s   Click Open (sidebar or titlebar)
3–6s   Folder picker → harbor (path truncated if personal)
6–10s  Land on Dashboard: project name · Now CTA · Targets populated
Hold   1s on useful state — then cut
```

**Do not show:** installer, Start Menu, license EULA, “working in …” empty forever, Output dock.  
**Must show:** one folder → instant orientation (name + Targets + Next action).

### Drop to site

5. Copy clips to `docs/assets/demo/v0.2.1/` and `apps/website/public/demo/v0.2.1/`.  
6. Point `/demo` at `v0.2.1` (or keep dual shelf).  
7. Update captions in this file if wording changes.

## Presenter line (say or subtitle)

> Ship Studio sequences the final mile — sign, release, deploy — and remembers where you are. Vendors and you still finish the irreversible steps.

## Success criteria

- [ ] **No install / setup** footage in any clip  
- [ ] T1 alone proves usefulness: folder → detected layout + clear next action  
- [ ] Core GIFs recorded on a real Desktop build (not a broken half-UI)  
- [ ] Final GIF size **720×420** (capture may be 960×560 then scaled)  
- [ ] No console flash during Verify / page switches  
- [ ] No claim of one-click deploy / auto OAuth / live publish  
- [ ] Fixture path only — no aperio secrets on screen  
