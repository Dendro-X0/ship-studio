# Desktop shell modularization — Band M1

**Status:** M1 shipped — leaf modules extracted from `main.ts`  
**Updated:** 2026-09-20  
**Owner:** `apps/desktop/src`  
**Parent:** maintainability · CodaCtrl-style module shelves (not a behavior change)

## Problem

| File | ~LOC | Issue |
|------|------|--------|
| `apps/desktop/src/main.ts` | 4200+ | God file — types, chrome, publish, portal, sidebar, icons |
| `apps/desktop/src/styles.css` | 2200+ | Same for CSS (later band) |
| `crates/shipctl/src/{publish,launch,config}.rs` | 2–3k each | Separate Rust bands — do not mix with Desktop M1 |

## M1 goal (this band)

Split **leaf** modules out of `main.ts` with **no behavior change**:

```
apps/desktop/src/
  types.ts              — shared TS types
  util.ts               — escapeHtml, path helpers, prettyMaybe
  icons.ts              — local SVG map (bundled public/icons)
  integrations-data.ts  — INTEGRATION_WIZARDS catalog
  constants.ts          — VIEW_META, ACTION_IDS, storage keys, RELATED labels
  main.ts               — state + UI apply + event wire-up (still large; shrink next)
```

## Non-goals (M1)

- Rewriting to React/shadcn  
- Splitting `publish.rs` / `launch.rs`  
- Changing shipctl JSON contracts  
- Circular “app context” framework

## Later bands (queued, not this PR)

| Band | Slice |
|------|-------|
| **M2** | `sidebar.ts` · `integrations-ui.ts` · `output-preview.ts` · `cmdk.ts` |
| **M3** | `publish-ui.ts` · `launch-ui.ts` · `portal-ui.ts` |
| **M4** | CSS partials (`styles/shell.css`, `styles/publish.css`, …) |
| **R1** | `shipctl` publish plan builder vs actions (Rust) |

## Proof

- [x] `pnpm --filter ship-studio-desktop build` (tsc + vite)  
- Desktop smoke: Targets · Integrations icons · Set up Polar  

## Claim

Modularized ≠ behavior-fixed. UI must match pre-split dogfood.

## M1 landed

| Module | Role |
|--------|------|
| `types.ts` | Shared shipctl JSON / UI types |
| `util.ts` | Pure string/path helpers |
| `icons.ts` | Bundled SVG map |
| `integrations-data.ts` | Payment/email wizard catalog |
| `constants.ts` | Nav meta · storage keys · action ids |
| `main.ts` | State · apply · wire-up (~3.7k, down from ~4.2k) |
