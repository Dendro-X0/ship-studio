# Ship Studio Desktop

CodaCtrl-shaped shell for the local Ship bridge.

## Run

```bash
cd "E:/Web Projects/ship-studio"
# One-shot release stage (builds shipctl + desktop, copies shipctl beside exe)
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

Dev:

```bash
cargo build -p shipctl --release
cd apps/desktop && npm install && npm run tauri dev
```

Shell: sidebar dashboard · Ctrl+K command search · Launch / Portal / Ritual / Tools views.

Bare `cargo build -p ship-studio-desktop` keeps `cfg(dev)` → localhost refused. Use `tauri build` / `tauri dev` / `stage-desktop.sh`.

Optional env: `SHIPCTL_PATH`, `SIGNET_PATH`, `ORBIT_PATH`.

## UI

- Sidebar nav + topbar search (Ctrl+K)
- One window = one repo; project bind in sidebar
- Dashboard health + quick actions; Launch / Portal / Ritual / Tools / Output views
- Auto Doctor on open; Offline / Include deploy toggles
- Live streamed output dock; **Cancel** kills process tree
- Ritual presets + editable `sign_args` / `deploy_args`
- Shortcuts: Esc · Ctrl+K · Ctrl+D · Ctrl+S · Ctrl+Enter · Ctrl+Shift+Enter
