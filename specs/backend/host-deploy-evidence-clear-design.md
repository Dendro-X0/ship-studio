# Host deploy evidence sync after remote delete — design

**Status:** Implement  
**Updated:** 2026-09-27  
**Parent:** [host-deploy-confirm-design](../frontend/host-deploy-confirm-design.md)

```text
GOAL:  After the operator deletes a Pages/site on the vendor dashboard, Studio can
       drop stale last-run Results / Cancel on dashboard without a redeploy.
NOT:   Polling Cloudflare APIs · auto-delete vendor projects · Studio-held tokens
```

## Problem

`.ship/last-run.json` stays `ok: true` with `*.pages.dev` URLs after the project is deleted. Pulse and Results keep showing success; Cancel on dashboard stays visible. Vendor UI shows Not found.

## Contract

| Control | Behavior |
|---------|----------|
| **Cancel on dashboard** (Results bay) | Open vendor deep link (unchanged) |
| **Clear evidence** (wizard primary cancel slot when cloud evidence exists) | Delete `.ship/last-run.json` for this project · refresh pulse · hide Results / Cancel |
| Toast after Cancel on dashboard | Action **Clear Studio evidence** |

Next **Deploy** for that host writes a new last-run and clears the dismiss.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `tsc --noEmit` (desktop) · `cargo check -p orbit-yard-desktop` if Tauri changed |
| L2 | After CF deploy evidence → Clear evidence → Results + Cancel hidden |
| L2 | Deploy again restores evidence |
