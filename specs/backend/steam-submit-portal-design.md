# Steam submit portal — band #29

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** SCOPE-OF-SERVICE · north star  
**Owner:** `publish` · `signpath`

## Product framing

Portal only. `listing.steam` stays the store-page gate. `submit.steam` opens official Steamworks depot/build docs. Human uploads. Bridge never calls the Steam API.

## Step

| Id | When | URL |
|----|------|-----|
| `submit.steam` | Advanced + Public + `detected.steam` | `https://partner.steamgames.com/doc/sdk/uploading` |

Local intent already drops `submit.*`. General does not include listing/submit.

## Acceptance

- [x] Advanced + `steam_appid.txt` includes `listing.steam` and `submit.steam`
- [x] General omits `submit.steam`
- [x] Sign portal includes `submit.steam` when steam detected

## Non-goals

- Steamworks API upload / steamcmd from the bridge
- Watching review status

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl steam_` |
