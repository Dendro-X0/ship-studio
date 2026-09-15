# Mobile archetype & store listing — first slice

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #1  
**Owner:** `scopes`, `config::probe`, `publish`, `signpath`
## Scope

- Detect Mobile (android / ios / Expo / Flutter / Capacitor)
- `ScopeKind::Mobile`
- Advanced publish steps: `listing.play`, `listing.app_store` (URL + Confirm)
- Sign portal: App Store Connect path (submission) alongside Apple certificates

## Non-goals this slice

- Automating Play/ASC upload or review  
- Steam / MS Store submission split (gap #2)  
- Mobile deploy providers (TestFlight CI, etc.)  

## Proof

- L1: fixture with `android/` + `app.json` (expo) yields Mobile scope + Advanced listing steps  
- L2: General mode omits `listing.play` / `listing.app_store`  
- L3: Desktop Sign shows App Store Connect when mobile/desktop  
