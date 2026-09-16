# Archetype launch matrix — band #21

**Status:** Done (first slice, 2026-09-16)  
**Parent:** `release-surface-map.md` · `SCOPE-OF-SERVICE.md`  
**Owner:** `config::probe`, `publish`, `pulse`, `assist`  

## Problem

Indie launches span Web/PWA/SaaS, desktop/mobile, API, OSS, games, digital products, and AI models — but the release-surface map only listed Api/Web/Desktop/Mobile/Container/Db. Operators need detection + honest sequencing per archetype without a second wizard.

## Scope (first slice)

### Document full archetype × phase matrix

See `release-surface-map.md` § Archetypes (extended) — maps each indie product type to Studio lanes vs human/vendor work.

### PWA detection

- Signals: `manifest.webmanifest` / `manifest.json` with PWA display, or `vite-plugin-pwa` in package.json  
- Pulse + Assist + probe hints (no fake deploy — host still Orbit/marketing)

### Hugging Face / AI model lane (opt-in + conservative auto)

- Opt-in: `.ship/markets` `hf` / `huggingface`  
- Auto: `modelcard.md` / `MODEL_CARD.md` / `.huggingface/`  
- Advanced `listing.huggingface` — portal URL + Confirm (no `huggingface-cli upload` from bridge)

## Non-goals

- HF weight upload automation  
- Replicate / Modal / RunPod lanes (until a real ship dogfoods one)  
- Game depot upload (SteamCMD)  
- PWA as separate deploy provider  

## Proof

- L1: PWA fixture → `detected.pwa` + pulse note  
- L1: HF fixture → `listing.huggingface` Advanced only  
- L2: dogfood script includes `listing.huggingface` when fixture markets include `hf`  
