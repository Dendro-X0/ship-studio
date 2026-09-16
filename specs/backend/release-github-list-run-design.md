# GitHub Release list Run — band #18

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md`  
**Owner:** `publish`, `adapters`  
**Updated:** 2026-09-15  

## Problem

`release.github` is Open `/releases/new` + Confirm only. Operators cannot see existing tags/releases from Publish before cutting a new one. `gh release list` is a safe read-only Run (mirrors `ci.release` / `gh run list`).

## Scope (first slice)

1. Advanced `release.github` Run: `gh release list --limit 5`  
2. Detail: Run list first; create draft/assets on the vendor UI; Confirm after published  
3. Verify: `gh release list --limit 1` non-empty → ok; empty → honest fail  
4. Doctor: when GitHub remote / CI release present, note gh for list Runs

## Non-goals

- `gh release create` from the bridge  
- Changing Signet `sign.self.release` path  

## Proof

- L1: fixture asserts `release.github.run` starts with `gh release list`  
- L2: `cargo test -p shipctl`  
