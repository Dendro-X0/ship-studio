# Marketplace submit parity (itch / Epic) — band #31

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** SCOPE-OF-SERVICE · extra-marketplaces · steam-submit  
**Owner:** `publish` · `signpath`

## Product framing

Mirror Steam: listing = store page; submit = build upload docs. Portal only — no butler/EAC API from the bridge.

## Steps

| Id | When | URL |
|----|------|-----|
| `submit.itch` | Advanced + Public + `itch` | `https://itch.io/docs/butler/` |
| `submit.epic` | Advanced + Public + `epic` | `https://dev.epicgames.com/docs/epic-games-store/` |

`listing.itch` / `listing.epic` stay; detail cues point at Submit next.

## Acceptance

- [x] Advanced + markets itch/epic → both listing and submit steps  
- [x] General omits  
- [x] Sign portal includes submit paths when detected  

## Non-goals

- Running butler / Epic Dev Portal upload from Studio  
- Watching review status  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl steam_and_markets` · `itch_epic_sign_portal` |
