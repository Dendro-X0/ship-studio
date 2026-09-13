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

Bare `cargo build -p ship-studio-desktop` keeps `cfg(dev)` → localhost refused. Use `tauri build` / `tauri dev` / `stage-desktop.sh`.

Optional env: `SHIPCTL_PATH`, `SIGNET_PATH`, `ORBIT_PATH`.

## UI

- Open folder (one window = one repo); last + recent projects
- Auto Doctor on open; title shows project name
- Offline / Include deploy (persisted)
- Workflow stepper + tool cards + detect chips + last run
- Live streamed output; **Cancel** kills process tree
- Ritual presets + editable `sign_args` / `deploy_args`
- Footer: resolved shipctl path + shortcuts
- Shortcuts: Esc · Ctrl+D · Ctrl+S · Ctrl+Enter · Ctrl+Shift+Enter
