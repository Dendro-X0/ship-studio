# Dashboard CTA no-op — investigation

**Status:** Implemented (fix) — capabilities rebuild + `tsc` clean; tauri reloaded with new opener allowlist  
**Evidence:** Operator screenshot — Start publish / Switch project no prompt; Output `opener.open_path not allowed. Permissions associated with this command: opener:allow-open-path`. Health tiles stuck on “Pulse after bind”.

## Root causes

1. **Opener capability** — `capabilities/default.json` only has `opener:default` (URLs / reveal). Sidebar `.ship` / Open path needs scoped `opener:allow-open-path`. Error shown in Output when Reveal fails — unrelated to CTAs but matches console.

2. **Switch project** — `#now-switch` calls `toggleProjectSwitcher(true)` without `stopPropagation`. Document click listener then closes the titlebar switcher on the same click → looks like “no prompt”.

3. **Start publish** — `runPulseAction` synthesizes `#btn-publish.click()`. Disabled buttons ignore click (busy/`running`). Concurrent `loadJsonCmd(["scopes"])` during bind races `refreshSessionNow` (`pulse`) via global `running` lock → pulse never applied → stale health; later CTA still fragile if busy.

## Fix plan

- Add Windows-friendly `opener:allow-open-path` allowlist (`/**`, `**`, drive roots).
- `stopPropagation` on Switch; keep titlebar switcher as the prompt.
- `runPulseAction`: call `refreshPublish` / peers directly; default empty view → `publish`.
- `bindProject`: await scopes (or after pulse) — no parallel `loadJsonCmd`.
- Bound identity: always set `data-pulse-id` / `data-pulse-view` defaults.

## Proof

- L1: `tsc --noEmit` in `apps/desktop`
- L3: Restart `tauri dev`; Switch opens switcher and stays open; Start publish navigates to Publish and loads plan; `.ship` opens Explorer without opener error
