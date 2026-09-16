# Pulse cut hints for mid-publish steps — band #17

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md`  
**Owner:** `pulse`  
**Updated:** 2026-09-15  

## Problem

Dashboard Now already prefers mid-publish Continue, but the cut hint only fired for `sign.*` / `trust.pack` / `ship.desktop_cut`. Operators on `ci.release`, `listing.npm`, `container.build`, or `release.github` got a generic “Open/Run…Confirm” with no step-specific cue.

## Scope (first slice)

1. Expand `publish_cut_hint(current_id)` for:
   - Signet cut (existing)  
   - `release.github`  
   - `ci.release`  
   - `listing.npm` / `listing.crates`  
   - `container.build` / `container.deploy`  
   - `legal.baseline` / `marketing.deploy` / `suite.url_sync`  

2. Keep mid-publish Continue as primary; hints are detail suffixes only.

## Non-goals

- New Now primary actions per step  
- Changing hard-block / already-live logic  

## Proof

- L1: unit — `ci.release` / `listing.npm` detail contains the new cue  
- L2: `cargo test -p shipctl`  
