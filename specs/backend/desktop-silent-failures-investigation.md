# Desktop silent failures — investigation

**Status:** Investigation → slice 1 implemented (working tree)  
**Updated:** 2026-09-25  
**Symptom:** Many Desktop actions fail with toast `«cmd» failed` / sticky **FAILED**, little or no actionable detail. Portal Cloudflare steps can load, then Login CLI still fails.

## Evidence

| Probe | Result |
|-------|--------|
| `shipctl portal --project fixtures/harbor --provider cloudflare` | Exit 0 · JSON steps OK |
| `shipctl portal … --provider orbit` | Exit 1 · `unknown provider 'orbit'` |
| Screenshot | Cloudflare steps visible + toast `portal failed` + Dock FAILED |
| CodaCtrl | workspaceAligned; no CDP session yet (preflight: need remote debugging) |

## Failure classes (not one-off bugs)

### F1 — Interactive CLI in headless `run_shipctl`

`Login CLI` calls `run(["portal", …, "--login"])` → Windows `CREATE_NO_WINDOW` + piped stdio. `wrangler login` / `vercel login` need a real terminal. Result: spawn/auth dies → exit ≠ 0 → toast **`portal failed`** with no guidance.

Same class as Launch/Publish Open (already fixed via `open_*_terminal`).

### F2 — Catalog provider IDs ≠ portal ProviderId

Platforms catalog sets `provider: "orbit"`. Portal parse rejects it. **Portal steps** / openPortalProvider → hard fail, often with only `portal failed`.

### F3 — Silent early returns after fail

```ts
if (!result?.ok || !result.stdout) return; // loadPortal, openPortalProvider, loadSecrets, …
```

`run()` already toasted a one-word failure; callers drop stderr. Operators see “failed” and a full portal list (from a prior success) — looks random.

### F4 — Sticky statusbar FAILED

`setBusy(false, "Failed", true)` for any non-ok exit, including expected human pauses and missing CLIs. Dock stays red until another success.

## Root fix (design → slice 1)

| Class | Fix |
|-------|-----|
| F1 | Login CLI → `open_portal_login_terminal` (interactive), never headless `--login` |
| F2 | Only pass portal-known providers; Orbit uses Open dashboard only (no Portal steps) |
| F3 | `failCmdToast(cmd, result)` always includes polished stderr / exit code; callers toast on null |
| F4 | Missing-CLI / unknown-provider → Ready + err toast (not sticky FAILED); keep FAILED for spawn/crash |

## Non-goals (this slice)

- Full CDP dogfood (Tauri needs `--remote-debugging-port`; separate attach recipe)
- Softening Confirm honesty
- Rewriting shipctl portal catalog
- Remaining Desktop reliability classes (Tools / Env Put / Ritual terminal parity) — track in [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md); design before more patches

## Remaining after slice 1

Superseded by the full reliability band: [desktop-reliability-investigation.md](./desktop-reliability-investigation.md) · [desktop-reliability-design.md](./desktop-reliability-design.md). Do not expand this file with drive-by fixes.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `pnpm exec tsc --noEmit` (desktop) |
| L2 | Harbor → Platforms Cloudflare → Portal steps loads; Login CLI opens terminal (not `portal failed`) |
| L2 | Platforms Orbit → Open dashboard works; Portal steps hidden |
| L2 | Fake fail still shows stderr snippet in toast |
