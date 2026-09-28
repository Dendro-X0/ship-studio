# Targets catalog parity — design (Slice 1)

**Status:** Slice 1 shipped (L1 tsc) — L2 Harbor dogfood  
**Updated:** 2026-09-27  
**Parent:** [studio-scopes-design](../backend/studio-scopes-design.md)

```text
GOAL:  Targets page reads as a first-class catalog (Deployment-like cards),
       not a sparse “Scopes” checkbox list.
NOT:   Per-target Deploy/Sign jumps · auto-detect-on-open · new shipctl fields
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

## Layout

- Keep `#scope-grid` / `#stage-scope-grid`; extend `.scope-card` styles.
- Empty: “No targets detected — press Detect.”
- Publish Stages inline uses the same `scopesGridHtml` / `fillScopeGrids`.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `npx tsc --noEmit` (desktop) |
| L2 | Harbor Targets: icons · path · selected tint · Detect/Save |
| L2 | Publish Stages scopes inline still toggles/saves |

## Sidebar

Single **Targets** item under Ship (with Dashboard / Publish / Sign / Deployment / Env). Per-target tree removed — selection lives on the Targets page.

## Follow-ups

- Slice 2: Open folder / jump Deployment  
- Slice 3: auto-detect on bind + dirty Save cue  
