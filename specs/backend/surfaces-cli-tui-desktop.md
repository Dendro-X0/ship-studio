# Surfaces — CLI + TUI + Desktop

**Status:** Active  
**Updated:** 2026-09-13  
**Owner:** `crates/shipctl` (CLI/TUI) · `apps/desktop` (Tauri)

## Goal

One shipping portal, **three surfaces**, same engine (`shipctl`):

| Surface | Role |
|---------|------|
| **CLI** | Scriptable / MCP / JSON (`doctor`, `portal`, `flow`, …) |
| **TUI** | Terminal wizard: pick providers, open entries, run login/configure/flow |
| **Desktop** | Same workflow in a window (CodaCtrl-shaped; invokes `shipctl`) |

## Invariants

1. Desktop and TUI are thin shells — no second business logic.
2. Portal semantics stay: navigate entry points; human does OAuth/env.
3. Bridge does not call vendor HTTPS; shells may open URLs / spawn login CLIs on operator action.

## TUI v2 (`shipctl tui`)

- Screens: Home · Providers (Space toggle) · Portal · Ship wizard (phased)
- Wizard: Doctor → Pick providers → Portal entries → Configure → Flow dry-run → Done
- Keys: ↑↓ · Enter · Space · `w` wizard · `o`/`l`/`a` in portal · `n` next · Esc back · `q` quit
- Non-TTY: refuse with hint to use JSON CLI

## Desktop v2

- Portal steps panel (per-step Open / Login CLI, provider filters, Open all)
- Workflow strip: Doctor → Portal → Sign → Deploy
- Same command names as TUI home actions

## Proof

- L1: `cargo test -p shipctl`
- L2: `shipctl tui --help`; `shipctl portal` JSON unchanged
- Desktop: Portal fills `#portal-panel` from shipctl JSON
