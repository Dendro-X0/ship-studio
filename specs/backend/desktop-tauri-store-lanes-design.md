# Desktop Tauri store lanes — band #23

**Status:** Done (first slice, 2026-09-16)  
**Parent:** `release-surface-map.md` · aperio Advanced dogfood  
**Owner:** `signpath`, `publish`  

## Problem

`aperio` is Desktop (Tauri) + Web (Vercel). Advanced Publish currently injects **Play Console / Android upload-key** steps because `signpath` treats `detected.tauri` as `wants_play`.

That is wrong: Tauri desktop ≠ Android / Play. Operators see noise gates that never apply.

## Evidence (aperio)

- Scopes: `desktop.desktop` (`apps/desktop/src-tauri`), `web.web` (Vercel)
- No `android/` / `ios/` / Expo
- Plan wrongly included `sign.official.android`
- Correct for desktop: Apple notarization, Windows Authenticode, optional MAS + Microsoft Store submit, Signet release, `ci.release` (`release-desktop.yml`)

## Scope (first slice)

1. **`wants_play`** — android / mobile / expo only (remove `tauri`)
2. Keep Tauri → Apple notarization, Windows certs, MAS submit, Microsoft Store submit
3. Unit: Tauri fixture must **not** get `official.android` / `submit.play`
4. Mobile fixtures still get Play lanes

## Non-goals

- MAS / MS Store opt-in via `.ship/markets` (possible later)
- Changing Web deploy / Orbit PATH messaging
- Auto-creating `signet.toml` for aperio

## Acceptance

- aperio Advanced plan has no `sign.official.android` / `submit.play` / `listing.play`
- Mobile fixture still has Play listing + submit
- `cargo test -p shipctl`
