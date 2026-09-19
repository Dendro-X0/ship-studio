# MCP publish watch — band #34

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** publish-progress-watch (#27) · tui-publish-watch (#33) · north star  
**Owner:** `mcp` · `publish`

## Product framing

Watch surface parity: CLI · Desktop · TUI · **MCP**. Agents poll `ship_publish_watch` (one local Verify probe per call). Never vendor HTTPS; never auto-Confirm unless `auto_confirm: true`.

## Tool

```text
ship_publish_watch { project, auto_confirm? }
→ { ok, message, step_id, finished, prompt, current_index, total, auto_confirmed }
```

Uses `publish::watch_probe` (JSON-safe; no stdout pollution of MCP stdio).

## Acceptance

- [x] Tool listed in MCP tools/list  
- [x] `watch_probe` returns structured JSON  
- [x] Unit test for probe  

## Non-goals

- Long-running MCP subscription / streaming watch  
- OS push notifications  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl watch_probe` |
| L2 | MCP `tools/call` ship_publish_watch (optional) |
