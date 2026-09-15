# Publish portal & wizard — design

**Status:** Implemented  
**Updated:** 2026-09-14  
**Owner:** `crates/shipctl` (`publish`) · Desktop Publish view

## Vision

A **manual publishing assistant**: open official entries or run local Signet/Orbit CLIs, operator finishes on vendor UI, Confirm → Next — through the full publish path in minutes. Not a cloud publisher; a sequenced portal.

## Sprint steps (adaptive)

| id | When | Open / Run | Done |
|----|------|------------|------|
| doctor | always | — | doctor ok |
| scopes | multi-dir | scopes UI | active scopes saved |
| env | paste hints | env portal / put | confirm after puts |
| oauth.* | detected providers | wrangler/vercel/… login | whoami / confirm |
| configure | always | shipctl configure | studio.json |
| sign.self.* | Tauri / signet.toml | signet scan/build/… | exit 0 / confirm |
| sign.official.* | sign_path ≠ self | Apple/MS/Play/GH URLs | confirm |
| listing.* | Polar | polar.sh dashboard | confirm |
| dry_run | always | flow --dry-run | plan ok |
| deploy.* | selected scopes | shipctl deploy | last-run / confirm |
| live_check | after deploy | open app URL if known | confirm |

## Commands

```text
shipctl publish [--project .]
shipctl publish open|run|verify|confirm|next|reset
```

State: `.ship/publish.json` (no secrets).

## Surfaces

- CLI JSON + prompts  
- Desktop **Publish** (primary): Open/Run · Confirm · Next · minutes strip  
- Assist remains a checklist overview; Publish is the live stepper

## Invariants

Same as guided-launch / studio-scopes: no vendor HTTPS from bridge; no secret values stored.

## Proof

- L1: wrangler fixture includes deploy; tauri includes self-sign  
- L2: `shipctl publish` / confirm / next  
- L3: Desktop Publish controls
