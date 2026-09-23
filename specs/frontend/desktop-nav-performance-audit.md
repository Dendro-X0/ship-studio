# Desktop nav freeze — performance audit

**Status:** Investigation + hot-path cut applied (L2 dogfood pending)  
**Owner:** `apps/desktop/src/main.ts` · `styles.css`  
**Surface:** Tauri WebView2 · aperio-class monorepos (many scopes + long Publish plan)  
**Symptom:** Switching sidebar pages freezes the window for several seconds.

## Method

Code-path audit of `setView` and shared shell (no flamegraph required for first cut).  
Dogfood context: aperio bound · Advanced · Public · Publish mid-flight (~19 steps) · ~12 Web targets in sidebar.

## Hot path (before cut)

```ts
function setView(id: string) {
  // toggle .view / .nav-item
  syncProjectIdentity();       // many DOM writes + syncNowQuick
  syncBackToPublish();
  if (id === "output") syncOutputMirror();
  if (id === "integrations") renderIntegrations();
  renderSidebarIntegrations(); // full innerHTML rebuild + rebind — EVERY nav
}
```

After cut: title/desc + `syncBackToPublish` only; integrations highlight in place; `renderSidebarIntegrations` at boot / rebuild only.

## Failure classes (ranked)

| # | Class | Evidence | Severity |
|---|--------|----------|----------|
| 1 | **Redundant sidebar rebuild** | `renderSidebarIntegrations()` on every `setView` tears down Payments/Email rows and rebinds listeners | High — pure waste |
| 2 | **Monolithic DOM** | All views live in one `index.html`; toggling `hidden` on large Publish/Portal/Dashboard trees forces style/layout with aperio-scale lists still in the document | High |
| 3 | **Identity sync on nav** | `syncProjectIdentity()` rewrites chrome/sidebar/dashboard CTA state on every switch | Medium |
| 4 | **Output dock weight** | Shared `#output` / Preview can hold large JSON; `syncOutputMirror` copies full text when opening Output | Medium when Output visited |
| 5 | **Console flash on spawn** | Windows GUI host + console `shipctl` without `CREATE_NO_WINDOW` → terminal pop + hitch on related-page loads | High on installed Windows — ✅ silent spawn |
| 6 | **Concurrent shipctl** | Publish Watch / mid-flight `run()` uses `setBusy` → `setProjectUi` loops all `ACTION_IDS` | Situational |

Not the primary story: Integrations wizard itself (only rebuilt when `id === "integrations"`). Signet/Orbit PATH. Polar KYC.

## Demo / release impact

Recording S1.1 GIFs against aperio Publish will show multi-second freezes and undercuts Solo positioning. **Park live GIF capture until nav hot path is fixed (or record against `fixtures/advanced-dogfood` / empty bind only as interim honesty).**

## Fix order (subtraction first)

1. **Stop rebuilding sidebar integrations on every nav** — ✅ `setView` highlights in place; rebuild only at boot / intent rebuild paths.  
2. **Slim `setView`** — ✅ title/desc + back-to-publish only (no `syncProjectIdentity`).  
3. **CSS:** ✅ `content-visibility` + `contain` on `.view`; animation only on `.view.active`.  
4. **Silent spawn:** ✅ Windows `CREATE_NO_WINDOW` on `run_shipctl` / `run_shipctl_env` / `git` / `taskkill`.  
5. **Output:** lazy mirror / truncate display buffer for UI (full log still in memory if needed).  
6. **Later:** virtualize sidebar targets when `scopes.length` is large; defer mounting inactive views.

## Proof plan

| Layer | Check |
|-------|--------|
| L1 | Code review: `setView` no longer calls `renderSidebarIntegrations` unconditionally |
| L2 | Manual: aperio bound · spam Dashboard ↔ Publish ↔ Env — target **&lt;100ms perceived** chrome response (no multi-second freeze) |
| L3 | Optional: Chromium Performance panel in WebView2 if freeze remains |

## Out of scope this band

- React rewrite · M2 modularization as the fix  
- Removing Adaptive Publish steps  
- Fake “instant” by skipping real Verify/Open work
