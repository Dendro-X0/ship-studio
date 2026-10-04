# Feature demo — v0.2.1 (Part 1)

**Status:** T1–T7 live on `/demo` (compressed Harbor GIFs)  
**Build:** App already installed ([v0.2.3](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.3)+) — **do not record setup/install**  
**Watermark:** `local app · recorded`  
**Replace:** stylized `docs/assets/demo/v0.1.0/` → live clips under `docs/assets/demo/v0.2.1/` (same file names when possible)  
**Shelf:** `apps/website` `DEMO_VERSION = v0.2.1` (T1–T7). Raw masters live in `_raw/` (gitignored) — never commit `.gif.gif` balloons.

## Demo stance

Prove **useful mid-flight shipping**, not “how to install software.” Everyone can install an app; the GIF shelf must show bind → detect → Continue/Open/Confirm in under a minute of watching.

**Honesty for Sign / Deploy clips:** T6 is **Open** only (official portal — no paid certs). T7 may show **one** streamed host **Deploy** → Results live URL → Open dashboard (H1–H5). No multi-host tour, notarization, store upload, or install/setup footage.

## Project choice

| Option | Use when | Verdict for Part 1 |
|--------|----------|--------------------|
| **`fixtures/harbor`** | Lightweight Desktop + Docs (+ Signet) — clean Targets for GIFs | **Primary — always** |
| `fixtures/advanced-dogfood` | Advanced multipath dogfood (mobile · CI · container · steam) | Engineering only — not Part 1 shelf |
| **Orbit Yard itself** | Engineering dogfood on the real monorepo | **Never for public GIFs** — already signed/deployed |
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
| 2 | **Targets** | Multi-surface detection (Web / API / Desktop cues in fixture) |
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
| Target weight | **≤1.5–3 MB** per clip | **10 fps** · ≤12 s · limited palette — raw 30–130 MB encodes are wrong settings |

### How to set it

1. Resize the Orbit Yard window to about **960×560** (titlebar included). Default app size is larger — shrink before recording.  
2. Record **only the app window** (no desktop wallpaper). Optional: cut **before** the browser tab fills the frame, or crop to Studio only.  
3. Export / convert to GIF at **720×420**, **10 fps**, **8–12 s** per clip.  
4. Rename once: `06-sign-open.gif` / `07-deploy-open.gif` — avoid `.gif.gif`.

Crop rule if the window must stay larger: keep **sidebar + Publish/Sign/Deployment chrome** in frame; drop the bottom Output dock if it steals height.

### ScreenToGif / encode recipe (stop the balloon)

GIF is a bad high-res container. **36 MB or encode Error** usually means capture FPS/size too high, or exporting GIF at capture resolution.

| Do | Don’t |
|----|--------|
| Record as **video** first (ScreenToGif → Video, or Win+G), then **File → Save as → GIF** | Record straight to GIF at 1080p / 30–60 fps |
| Trim to **≤12 s** before encode (T6/T7 shot lists) | Leave idle “Serving · Ns” / dashboard Not found in frame |
| Resize **during save** to **720×420** | Ship the full-window capture as the GIF |
| **10 fps** · palette **128–256** · enable **detect unchanged pixels** | 5 fps (choppy) or 24+ fps (huge) |
| Target **≤3 MB**; if over, drop to 8 fps or 128 colors | Chase “dashboard sharp in 1080p GIF” — `/demo` only shows ~720 CSS px |

**ffmpeg fallback** (after a short `.mp4` of the Studio window):

```bash
ffmpeg -y -i input.mp4 -vf "fps=10,scale=720:420:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" -loop 0 06-sign-open.gif
```

If still >3 MB: shorten the clip, hide Output dock, avoid full-screen browser (cut as the tab opens).

**Perception tip:** UI demos look smoother at **10 fps + clean motion** than at 5 fps “to save size.” Size comes from resolution × duration × unique pixels — not from “looking sharp for later zoom.”

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
| **T6** | `06-sign-open.gif` | `sign-open…` | Ship → **Sign** → Open one official portal → cut before paying | Studio opens the store / cert site — you finish there. |
| **T7** | `07-deploy-open.gif` | `Cloud-hosted…` | Cloudflare **Deploy** → Results live URL → **Open dashboard** (≤12 s mid-clip) | Deploy streams wrangler on this machine; Results show the live URL — Open dashboard to finish on Cloudflare. |
| *(extra)* | `08-selfhost.gif` | `selfhost…` | Optional local check — **not** on `/demo` shelf | — |

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

### T7 — Deployment Deploy + Open dashboard (one provider)

```text
Prep    Harbor bound · intent Public for this clip · Deployment · Cloudflare (or Orbit) selected
0–2s    Sidebar → Deployment
2–5s    ONE host card selected
5–10s   Deploy → stream briefly → Results live URL visible
10–12s  Open dashboard (or hold Results) — cut. No second host.
```

**Do not show:** multi-host tour, Docs/Learn more as the hero click, claiming every host is one-click.

Part 1 still never shows install / setup. T6 stays Open-only (no paid certs).

### Operator checklist — T6 / T7 only

```text
1. scripts/harbor-reset (or wipe fixtures/harbor/.ship)
2. Desktop bind Harbor · Advanced · Local; window ~960×560; Output dock hidden
3. T6: Sign → Open ONE official portal → cut before paywall → optional ← Back
4. T7: intent Public · Deployment → ONE host → Deploy → Results URL → Open dashboard (≤12s)
5. Export 720×420 ≤3 MB → docs/assets/demo/v0.2.1/06-sign-open.gif + 07-deploy-open.gif
6. Mirror apps/website/public/demo/v0.2.1/ · rows already in apps/website/src/lib/demo.ts
```

### Drop to site

1. Compress / rename → `docs/assets/demo/v0.2.1/01-bind.gif` … `07-deploy-open.gif` (masters in `_raw/`, gitignored).  
2. Mirror under `apps/website/public/demo/v0.2.1/`.  
3. `/demo` uses `DEMO_VERSION = v0.2.1` with T1–T7 rows in `demo.ts`.  
4. Optional `08-selfhost.gif` stays off-shelf (local check only).

## Presenter line (say or subtitle)

> Orbit Yard sequences the final mile — sign, release, deploy — and remembers where you are. Vendors and you still finish the irreversible steps.

## Success criteria

- [x] **No install / setup** footage in any clip  
- [x] T1 alone proves usefulness: folder → detected layout + clear next action  
- [x] T3 = publish check / Confirm — **no** paid signing in that clip  
- [x] T6 = Sign Open only (no cert purchase)  
- [x] T7 = one-host Deploy → Results URL → Open dashboard (H1–H5 honesty)  
- [x] Final GIF size **720×420** (T1–T7 mirrored under `public/demo/v0.2.1/`)  
- [ ] Optional: recompress any clip still ≫3 MB (`03-confirm-next` ~2.9–3.0 MB)  
- [ ] Spot-check: no long Serving / Not found / console flash in T6–T7  
- [x] No claim of auto OAuth / silent publish  
- [x] Fixture path only — no aperio secrets on screen  
