# Desktop Open/Related dashboard parity — band #16

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md` · adaptive-doctor-verify non-goal follow-up  
**Owner:** `apps/desktop`  
**Updated:** 2026-09-15  

## Problem

Publish **Related** already opens Dashboard for `desktop_view=dashboard` (`ci.release`, `legal.baseline`, `release.github`, `suite.url_sync`, `live_check`). **Open** only navigated studio detail for scopes/env/sign/portal/ritual — dashboard (and tools/launch) were excluded, so Open only fired `entry_url` and left the operator on Publish.

## Scope (first slice)

1. Treat any `desktop_view` in `RELATED_VIEW_LABELS` as studio detail on Open (same as Related targets).  
2. Keep existing behavior: after navigating, still `publish open` / terminal when `run` or oauth/sign/deploy.  
3. Spec + dogfood matrix note.

## Non-goals

- New Desktop views  
- Changing Related button logic  
- Auto-Confirm  

## Proof

- L1: `studioDetail` includes all `RELATED_VIEW_LABELS` keys (code)  
- L2: Desktop Advanced on dogfood — Open on `legal.baseline` / `ci.release` lands Dashboard + URL  
