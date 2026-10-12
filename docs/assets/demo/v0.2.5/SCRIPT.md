# Feature demo — v0.2.5 shelf (R4)

**Status:** **Live on `/demo`** — Harbor GIFs recorded on Desktop 0.2.5 (CDP capture)  
**Truth:** Download / CURRENT = **[v0.2.5](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.5)**  
**Build:** App already installed (**0.2.5**) — **do not record setup/install**  
**Watermark:** `local app · recorded`  
**Shelf:** `DEMO_VERSION = v0.2.5` · files under `docs/assets/demo/v0.2.5/` + `apps/website/public/demo/v0.2.5/`  
**Parent:** [presentation-readiness](../../../specs/backend/presentation-readiness-design.md) R4 · prior [v0.2.1 SCRIPT](../v0.2.1/SCRIPT.md)

## Demo stance

Prove **Get-live mid-flight** on CURRENT UI: bind Harbor → Open/Confirm → Sign Open → **Deploy → live URL**.

## Project

**`fixtures/harbor` only.** Reset before record: `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1`  
Advanced + Local for T1–T6; **Public** for T7 Deploy. Never record on the ship-studio monorepo.

## Capture

| Stage | Size |
|--------|------|
| Window | ~960×560 · Output dock hidden |
| Export | **720×420** · **10 fps** · ≤12 s · ≤3 MB |

ffmpeg (from short `.mp4` in `_raw/`):

```bash
ffmpeg -y -i _raw/01-bind.mp4 -vf "fps=10,scale=720:420:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" -loop 0 01-bind.gif
```

## Beats → files

| Task | Clip | Action (~8–12s) |
|------|------|-----------------|
| **T1** | `01-bind.gif` | Empty Dashboard → Open → harbor → Targets |
| **T2** | `02-open.gif` | Open surface / Login CLI (or skip if T7 covers) |
| **T3** | `03-confirm-next.gif` | Publish check → Confirm / Continue (no live deploy) |
| T4 | `04-output-preview.gif` | Statusbar Preview |
| T5 | `05-midflight.gif` | Mid-flight → Sign/Deployment → ← Back |
| **T6** | `06-sign-open.gif` | Sign → Open one official portal → cut |
| **T7** | `07-deploy-open.gif` | Cloudflare **Deploy** → Results URL → Open dashboard |

## Drop to site

1. Export GIFs into this folder (masters in `_raw/`, gitignored).  
2. Mirror → `apps/website/public/demo/v0.2.5/`.  
3. Set `DEMO_VERSION = "v0.2.5"` in `apps/website/src/lib/demo.ts`.  
4. `/demo` copy: shelf matches CURRENT (drop “Download is v0.2.4” honesty line).  
5. R5: keep “Download is the full OSS tool” on home (already present).

## Operator checklist

```text
1. harbor-reset (done when session prepped)
2. Desktop 0.2.5 · bind Harbor · Advanced · Local · window ~960×560
3. Record T1–T7 (ScreenToGif video→GIF or Win+G → ffmpeg)
4. Mirror public/demo/v0.2.5 · flip DEMO_VERSION
5. pnpm website:build · hostdeploy · spot-check /demo
```

## Success

- [x] No install footage  
- [x] T7 Deploy clicked on Harbor · Cloudflare (STRUCTURE OK / stream)  
- [x] 720×420 · shelf `v0.2.5` · CURRENT label honest  
- [x] Watermark `local app · recorded`  
- Capture helper: `scripts/r4-demo-capture.py` (WebView2 CDP screenshots → GIF)  
