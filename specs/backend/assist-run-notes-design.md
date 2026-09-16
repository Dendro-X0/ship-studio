# Assist notes for final-mile Runs — band #19

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md`  
**Owner:** `assist`  
**Updated:** 2026-09-15  

## Problem

Assist notes still described CI as “confirm Actions” and omitted registry dry-run / `gh release list` Runs shipped in bands #14–#18. Checklist overview lagged the Publish spine.

## Scope (first slice)

1. Update CI Assist step detail + note: Run `gh run list` after tag.  
2. Notes when detected:
   - `npm_publish` / `crates_publish` → dry-run Runs  
   - GitHub remote without Signet desktop: `release.github` → `gh release list`  
3. Keep container note (already accurate).  

## Non-goals

- New Assist steps / second wizard  
- Changing Assist step order  

## Proof

- L1: CI fixture note mentions `gh run list`  
- L1: npm+crates fixture notes mention dry-run  
- L2: `cargo test -p shipctl`  
