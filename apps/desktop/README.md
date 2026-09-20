# Ship Studio Desktop

CodaCtrl-shaped shell for the local Ship bridge.

## Run (pnpm — preferred)

From the **repo root**:

```bash
pnpm install
pnpm dev                 # Tauri window + Vite on :1420
# pnpm dev:ui            # Vite only (no native shell)
```

VS Code / Cursor: **Run and Debug → “Desktop: pnpm dev”**.

## Release stage

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

Or: `pnpm desktop:release` from the repo root.

Bare `cargo build -p ship-studio-desktop` keeps `cfg(dev)` → localhost refused. Use `pnpm dev` / `tauri build` / `stage-desktop.sh`.

Optional env: `SHIPCTL_PATH`, `SIGNET_PATH`, `ORBIT_PATH`.

## UI

- Sidebar nav + topbar search (Ctrl+K)
- One window = one repo; project bind in sidebar
- Dashboard health + quick actions; Launch / Portal / Ritual / Tools / Output views
- Auto Doctor on open; Offline / Include deploy toggles
- Live streamed output dock; **Cancel** kills process tree
- Ritual presets + editable `sign_args` / `deploy_args`
- Shortcuts: Esc · Ctrl+K · Ctrl+D · Ctrl+S · Ctrl+Enter · Ctrl+Shift+Enter
