# In-app update check — design (S0.8)

**Status:** Shipped (L1 tsc)  
**Updated:** 2026-09-27  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S0.8 · [desktop-windows-installer-design](./desktop-windows-installer-design.md)

```text
GOAL: Notice a newer GitHub Release; human Confirm opens the download page.
NOT: Silent auto-install · force update · updater plugin · delta patches.
```

## Behavior

| Trigger | Behavior |
|---------|----------|
| Boot (once, ~4s after paint) | Quiet — toast **only** if newer than running version and not snoozed |
| CmdK **Check for updates** / Tools button | Always report: newer · up to date · or check failed |
| Offline toggle on | Skip boot check; manual says “Offline — turn off Offline to check” |

## Compare

- Local: `@tauri-apps/api/app` `getVersion()` (matches Cargo / package `0.2.3`)
- Remote: `GET https://api.github.com/repos/Dendro-X0/ship-studio/releases/latest` → `tag_name`
- Semver `major.minor.patch` after stripping leading `v`

## CTA

- Toast action **Open download** → `Dendro-X0/ship-studio/releases/latest` (opener)
- Optional **Later** → snooze that remote tag in `localStorage` until a different tag appears

## Non-goals

- `tauri-plugin-updater` / MSI silent replace  
- Checking every hour in background  
- Channel / prerelease selection (latest release only)

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apps/desktop` `tsc --noEmit` |
| L2 | CmdK Check for updates — up-to-date toast on current cut; Open download opens GitHub |
