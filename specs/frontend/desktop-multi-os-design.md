# Desktop multi-OS (Linux · macOS · Windows) — design

**Status:** Active · 2026-10-15  
**Product:** Orbit Yard Desktop (Tauri v2)  
**Supersedes OOS note in** [desktop-windows-installer-design](./desktop-windows-installer-design.md) (S0.7 macOS/Linux installers).

## Goal

One Desktop codebase ships installable artifacts on **Windows · macOS · Linux**, same Adaptive Publish spine. Releases stop being Windows-only zip/NSIS.

## Non-goals

- Store notarization / Apple Developer ID / Microsoft Store (still human graduate lanes)
- Cross-compile from one host (CI builds natively per OS)
- Re-open Solo commercial checkout

## Architecture

| Piece | Rule |
|-------|------|
| Sidecar | Stage `resources/orbityard-bin.exe` (one name — Windows CreateProcess + Unix exec; avoids `orbityard`↔`.exe` collision) |
| Bundle targets | `"all"` — Tauri emits platform-native installers (NSIS / DMG·app / deb·AppImage) |
| Terminal open | Windows: wt → cmd; macOS: Terminal via osascript; Linux: common `-e` terminals |
| CI | Matrix `windows-latest` · `macos-latest` · `ubuntu-latest` → stage + `tauri build` → artifacts |
| npm | No hard-pinned win32-only `@tauri-apps/cli-*` / `@rollup/rollup-win32-*` |

## Proof

- L1: `pnpm desktop:installer` succeeds on each OS (or CI job green)
- L2: App launches; `resolve_orbityard` finds sidecar; Login CLI / terminal open works
- L3: Release assets named per OS (human `gh release` attach for first cut)

## Atomic steps

1. Sidecar + `tauri.conf` targets + stage scripts  
2. Unix `open_orbityard_terminal`  
3. Drop win32-hard npm deps  
4. CI desktop matrix + docs / handoff  
