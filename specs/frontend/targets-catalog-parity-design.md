# Targets catalog parity — design

**Status:** Slice 1–3 shipped (L1 tsc) — L2 Harbor dogfood  
**Updated:** 2026-09-28  
**Parent:** [studio-scopes-design](../backend/studio-scopes-design.md)

```text
GOAL:  Targets page reads as a first-class catalog (Deployment-like cards),
       not a sparse “Scopes” checkbox list.
NOT:   Per-target Deploy Run · new shipctl fields
```

## Product naming

| Surface | Copy |
|---------|------|
| Page title (`VIEW_META`) | **Targets** |
| Panel label | **Targets** |
| Hint | Active publish surfaces in this repo. Toggle what you ship; Save, then Confirm on Publish. |
| Sidebar | **Targets** (single Ship nav item — same pattern as Sign / Deployment) |
| Cmdk | Open Targets |
| View id | stays `scopes` (no nav break) |

## Card anatomy

| Element | Source |
|---------|--------|
| Kind icon | `kind` → web / api / desktop / docs / mobile / container / root |
| Title | `label` |
| Meta | `kind · relative` |
| Chips | `provider` + up to 2 `signals` |
| Selection | checkbox; checked = green border/tint (`.int-card.is-active` parity) |
| Actions (Slice 2) | **Folder** → reveal target dir · **Deploy** → Deployment catalog (provider host when known) |

## Layout

- Keep `#scope-grid` / `#stage-scope-grid`; extend `.scope-card` styles.
- Empty: “No targets detected — press Detect.”
- Publish Stages inline uses the same `scopesGridHtml` / `fillScopeGrids`.
- Card is a `div` with an inner select `label` so Folder/Deploy buttons do not toggle the checkbox.

## Slice 2 behavior

| Action | When | What |
|--------|------|------|
| Folder | Always (bound project) | `openPath(project/relative)` — `.` opens project root |
| Deploy | Always | `openPlatformsCatalog({ selectId })` — map `provider` → host id; else preferred/selfhost |

## Slice 3 behavior

| Surface | When | What |
|---------|------|------|
| Bind | Project bound | Quiet `scopes` detect (same as Detect) — cards fill without a manual Detect click |
| Dirty Save | Checkbox selection ≠ saved `active` | Save buttons → **Save changes** + `#scopes-dirty-cue`; clear after Save / re-detect |

## Proof

| Layer | Check |
|-------|--------|
| L1 | `npx tsc --noEmit` (desktop) |
| L2 | Harbor Targets: icons · path · selected tint · Detect/Save |
| L2 | Folder opens the target path; Deploy opens Deployment on matching host |
| L2 | Bind fills Targets without Detect; toggle shows dirty Save cue |
| L2 | Publish Stages scopes inline still toggles/saves |
| L2 (CLI 2026-09-28) | Harbor scopes detect/set · folder paths exist · nested parent git ignored — [evidence](../../docs/handoffs/evidence-harbor-s115-s116.md) |

## Sidebar

Single **Targets** item under Ship (with Dashboard / Publish / Sign / Deployment / Env). Per-target tree removed — selection lives on the Targets page.
