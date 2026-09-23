# Feature demo — v0.2.1 (Part 1)

**Status:** Recording plan for S1.1 live GIFs  
**Build:** [v0.2.2 installer](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.2) (Confirm/Next gate fix — prefer over v0.2.1)  
**Watermark:** `local app · recorded`  
**Replace:** stylized `docs/assets/demo/v0.1.0/` → live clips under `docs/assets/demo/v0.2.1/` (same file names when possible)

## Project choice

| Option | Use when | Verdict for Part 1 |
|--------|----------|--------------------|
| **`fixtures/advanced-dogfood`** | Controlled multi-surface layout (web · worker · signet · CI · steam stub) without personal secrets | **Primary — use this** |
| **Ship Studio itself** | Real monorepo (Desktop + website + shipctl); meta “ship the shipper” | Optional B-roll only — confusing for strangers |
| **aperio** | Extreme Targets list / Advanced Public mid-flight | **Avoid for public GIFs** (personal · PAUSED L4 · noise) |

**Rule:** Bind `fixtures/advanced-dogfood` from the installed Desktop. Prefer **Advanced + Local** for the spine (no fake live deploy). Use **Public** only for a short Polar Open beat if you want commerce honesty without charging.

## What Part 1 proves (one sentence)

Studio keeps **mid-flight order** across surfaces: you Open vendor doors, Confirm gates, and advance — a chatbot checklist cannot hold that state.

## Features to showcase (in)

| # | Feature | Why it sells |
|---|---------|--------------|
| 1 | **Bind project** | Local-first; one window = one repo |
| 2 | **Targets sidebar** | Multi-surface detection (Web / API / Desktop cues in fixture) |
| 3 | **Publish portal** | Step N/M spine — doctor → scopes → env → sign → … |
| 4 | **Open / Run** | Browser/terminal for the *human* gate (Studio does not OAuth) |
| 5 | **Verify → Confirm → Next** | Honest gatekeeping; state advances only when you Confirm |
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

### Prep

1. Install **v0.2.2** from GitHub Releases (clean install; quit any older Desktop).  
2. Reset fixture state if needed: delete `fixtures/advanced-dogfood/.ship/` (or bind fresh).  
3. Set capture size per **Capture resolution** above; dark theme as shipped; hide personal paths if possible (`…\fixtures\advanced-dogfood`).  
4. Capture tool: silent GIF or short MP4→GIF; no voiceover required for `/demo` shelf.

### Beats → files (keep names stable for `/demo`)

| Task | Clip file | Action on screen (~8–15s) | On-page caption |
|------|-----------|---------------------------|-----------------|
| T1 | `01-bind.gif` | Open Desktop → Open folder → select `advanced-dogfood` → Dashboard shows project | Pick a local folder. Studio stays on your machine. |
| T2 | `02-open.gif` | Publish → show step list → **Open / Run** (browser or terminal) → return to Studio | Open the vendor door. Studio does not OAuth for you. |
| T3 | `03-confirm-next.gif` | **Verify** (optional) → **Confirm** → **Next** (1–2 gates) | You confirm each gate. Studio advances the spine. |
| T4 | `04-output-preview.gif` | Open Output / Preview on plan or doctor JSON | Inspect local artifacts before the next Confirm. |
| T5 *(new)* | `05-midflight.gif` | From Publish mid-flight → Env or Sign → **Back to Publish** still on step N/M | Mid-flight state survives. A pasted checklist cannot. |

### Drop to site

5. Copy clips to `docs/assets/demo/v0.2.1/` and `apps/website/public/demo/v0.2.1/`.  
6. Point `/demo` at `v0.2.1` (or keep dual shelf).  
7. Update captions in this file if wording changes.

## Presenter line (say or subtitle)

> Ship Studio sequences the final mile — sign, release, deploy — and remembers where you are. Vendors and you still finish the irreversible steps.

## Success criteria

- [ ] All four core GIFs (plus optional mid-flight) recorded on **installer** build, not `pnpm dev`  
- [ ] Final GIF size **720×420** (capture may be 960×560 then scaled)  
- [ ] No console flash during Verify / page switches  
- [ ] No claim of one-click deploy / auto OAuth / live publish  
- [ ] Fixture path only — no aperio secrets on screen  
