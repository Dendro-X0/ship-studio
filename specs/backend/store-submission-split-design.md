# Official store submission split — gap #2

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #2  
**Owner:** `signpath`, `publish`

## Problem

Official vendor work conflates three jobs:

1. **Certificates / signing** — Apple Developer certs · Authenticode · Play upload key  
2. **Listing** — store presence / metadata (already: `listing.play` / `listing.app_store` / Polar)  
3. **Submission** — upload build + send for review  

Operators need distinct Advanced steps so Confirm advances one human gate at a time.

## Scope (first slice)

- Sign portal: keep certs paths; add/clarify **submission** paths (ASC already; add Microsoft Store apps; Play “Publish” / production track).  
- Advanced publish (Desktop / Mobile only):
  - `submit.app_store` — App Store Connect review (when ios / expo / mobile / tauri)  
  - `submit.play` — Play Console production / review (when android / expo / mobile)  
  - `submit.microsoft` — Partner Center app submission (when tauri / desktop)  
- General mode omits all `submit.*` (same as `listing.*`).  
- Do not automate upload or review APIs.

## Non-goals

- Steam (gap #6)  
- CI TestFlight upload  
- Changing General spine  

## Proof

- L1: Mobile fixture Advanced has `listing.*` **and** `submit.play` / `submit.app_store`  
- L1b: Tauri fixture Advanced has `submit.microsoft`  
- L2: General omits `submit.*`  
- L3: Sign portal JSON includes distinct cert vs submission URLs  
