# Project recents history

**Status:** Active — Desktop slice (enhance existing MRU)  
**Updated:** 2026-10-09  
**Owner:** Desktop bind / sidebar / project switcher  
**Parents:** [founder-minimal-spine](./founder-minimal-spine-design.md) (F5 local recents)

## Guarantee

Local folders the operator opens are remembered (MRU, cap **`MAX_RECENT` = 32**). Sidebar shows **up to 3 equal-width** rows (`RECENT_PILL_MAX` = 3). Full list + **search** live in a **Project history** dialog (hidden until **All projects…**). Entries persist in `localStorage` until **manual remove** or the folder is **gone** — pruned on load and on failed bind.

## Behavior

| Event | Action |
|-------|--------|
| Successful `bindProject` | Push path to front of recents |
| Click recent / switcher | Bind if dir exists; else remove + toast |
| **All projects…** | Open history dialog · filter by name/path |
| Startup | Prune missing paths · restore last project only if still a dir |
| Remove (×) | Drop from list without binding |

No cloud hub. Paths stay absolute local folders only.
