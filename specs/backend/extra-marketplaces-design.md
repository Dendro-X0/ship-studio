# Steam / extra marketplaces — gap #6

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #6  
**Owner:** `config::probe`, `publish`, `pulse`, `assist`

## Problem

Desktop/game ships often list on Steam, itch.io, or Epic after Signet. Studio only sequenced Polar + mobile/MS stores.

## Scope (first slice)

**Opt-in detection** (avoid false positives):

| Market | Markers |
|--------|---------|
| Steam | `steam_appid.txt`, `STEAM_*` env keys, or `.ship/markets` / `markets.json` containing `steam` |
| itch.io | `itch.toml`, `.itch/` dir, or markets file `itch` |
| Epic | markets file `epic` / `egs` only |

**Advanced publish** (URL + Confirm → portal):

- `listing.steam` → https://partner.steamgames.com/  
- `listing.itch` → https://itch.io/dashboard  
- `listing.epic` → https://dev.epicgames.com/portal  

General omits all `listing.*`. Pulse/Assist notes when any market detected.

## Opt-in how-to

```text
# .ship/markets (one per line)
steam
itch
epic
```

Or `.ship/markets.json`: `["steam","itch","epic"]`.

## Non-goals

- Steamworks upload / depot builds  
- Butler push automation  
- New ProviderId / OAuth for these markets  

## Proof

- L1: fixture with `steam_appid.txt` → `detected.steam`  
- L2: Advanced has `listing.steam`; General omits  
- L3: `.ship/markets.json` with itch + epic yields those listing steps  
