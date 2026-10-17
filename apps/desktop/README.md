# Orbit Yard Desktop

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

Windows · macOS · Linux (Tauri v2 — current host only; CI matrix builds each OS):

```bash
bash scripts/stage-desktop.sh              # portable binary + sidecar
bash scripts/stage-desktop.sh --installer  # + NSIS / DMG / deb·AppImage
# Windows:  ./target/release/orbit-yard-desktop.exe
# Unix:     ./target/release/orbit-yard-desktop
```

Or: `pnpm desktop:release` / `pnpm desktop:installer` from the repo root.  
Design: [desktop-multi-os-design](../../specs/frontend/desktop-multi-os-design.md).

Bare `cargo build -p orbit-yard-desktop` keeps `cfg(dev)` → localhost refused. Use `pnpm dev` / `tauri build` / `stage-desktop.sh`.

Optional env: `ORBITYARD_PATH`, `SIGNET_PATH`, `ORBIT_PATH`.

## UI

- Sidebar nav + topbar search (Ctrl+K)
- One window = one repo; project bind in sidebar
- Dashboard health + quick actions; Launch / Portal / Ritual / Tools / Output views
- Auto Doctor on open; Offline / Include deploy toggles
- Live streamed output dock; **Cancel** kills process tree
- Ritual presets + editable `sign_args` / `deploy_args`
- Shortcuts: Esc · Ctrl+K · Ctrl+D · Ctrl+S · Ctrl+Enter · Ctrl+Shift+Enter
