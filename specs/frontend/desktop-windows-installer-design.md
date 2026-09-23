# S0.7 — Windows installer (direct download)

**Status:** Shipped on [v0.2.0](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.0)  
**Owner:** `apps/desktop/src-tauri/tauri.conf.json` · `scripts/stage-desktop.sh --installer`  
**Asset:** `ship-studio-v0.2.0-windows-x64-setup.exe` (NSIS · currentUser · `shipctl` in `resources/`)

## Product rules

- Installer places **Desktop + `shipctl` sidecar** in the same install dir (Desktop already resolves sibling `shipctl.exe`).
- WebView2: use download bootstrapper (smaller installer; online first install OK for Solo).
- Prefer **NSIS** (`-setup.exe`) over MSI for stranger-facing “download and run” UX.
- Signing / SmartScreen: out of band for this cut (no false claims); ship unsigned NSIS like the portable zip.
- Do **not** claim in-app update check in this band (S0.8).

## Config slice

```json
"bundle": {
  "active": true,
  "targets": ["nsis"],
  "resources": { "../../../target/release/shipctl.exe": "shipctl.exe" },
  "windows": {
    "nsis": { "installMode": "currentUser" }
  }
}
```

Alternative if resources path is awkward: copy `shipctl.exe` into `src-tauri/binaries/` (or `resources/`) before `tauri build`, and reference that path.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `tauri build` produces `*-setup.exe` under `target/release/bundle/nsis/` |
| L2 | Installer uploaded to `v0.2.0` · Releases page lists setup.exe |
| L3 | Fresh install: Desktop starts · status bar resolves `shipctl` beside install dir |

## Out of scope

- Code signing · auto-update · MSI · macOS/Linux installers
