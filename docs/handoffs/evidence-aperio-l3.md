# Evidence — aperio Advanced Publish (band #23 L3)

**Date:** 2026-09-18  
**Subject:** `E:/Web Projects/aperio` (Desktop Tauri + Web Vercel)  
**Binary:** `target/release/shipctl.exe` + `target/release/ship-studio-desktop.exe`

## CLI (subject plan)

| Check | Result |
|-------|--------|
| No `sign.official.android` / `submit.play` / `listing.play` | Pass |
| Sign portal: no `official.android` / `submit.play` | Pass |
| Keeps Apple / Windows / MAS / MS Store | Pass |
| Mobile dogfood still has `submit.play` | Pass (`fixtures/advanced-dogfood`) |

Cut slice: `sign.self.scan` → `build` → `trust.pack` → `release_dry` → official Apple/Windows/ASC → `sign.self.release` → `submit.app_store` → `submit.microsoft` → `ci.release` → `dry_run` → `deploy.web.web` → `live_check`

Artifact: `docs/handoffs/evidence-aperio-l3-cli.json`

## Desktop L3 (UI)

| Check | Result |
|-------|--------|
| Bound project | aperio |
| Mode | Advanced |
| Open/Run | Pass (doctor / publish spine) |
| Refresh plan | **20 steps** visible |
| Play / Android noise | **None** (`hasSubmitPlay: false`) |
| MAS / Microsoft Store | Present |

Current at capture: Step 3/20 — legal baseline (LICENSE · SECURITY.md missing — operator gate).

Artifacts:
- `docs/handoffs/evidence-aperio-l3-desktop.json`
- `docs/handoffs/evidence-aperio-l3-desktop.png`

## Env notes

- First Tauri build failed on corrupted cargo `cc-1.4.5` (missing `src/target/*`); registry entry removed, rebuild succeeded.
- WSL `bash` unavailable on this host; dogfood assertions run via PowerShell + release `shipctl`.
