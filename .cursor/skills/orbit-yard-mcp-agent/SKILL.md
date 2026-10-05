---
name: orbit-yard-mcp-agent
description: >-
  Guide Orbit Yard release work via orbityard MCP (yard_* tools): next publish
  gate, exact vendor URLs, env Put launch, local Verify. Use when shipping,
  deploying, signing, putting secrets, or validating an Orbit Yard / orbityard
  project — prefer MCP over Desktop Integrations wizards.
---

# Orbit Yard — MCP agent guide

Client stays thin. You assist through **`orbityard mcp`** tools. You do **not** hold secrets, finish OAuth, create vendor API keys, or upload store binaries.

## Minutes-to-launch loop

```text
1. yard_setup | yard_publish     → phase + human action + next tools
2. yard_publish_open             → browser and/or Login CLI (visible TTY)
3. Human: create key / paste Put / Confirm on Client or CLI
4. yard_env_put { spawn: true }  → NAME only — never value
5. yard_publish_verify | watch   → local Verify only
6. Human Confirm → yard_publish_confirm → yard_publish_next
```

Prefer **`yard_setup`** first when the operator asks to launch or finish setup — it returns `orbit-yard/setup/v1` with `human.action`, `entry_url`, optional `secret_name`, and `agent.next_tools`.

## Tools (prefer these)

| Need | Tool |
|------|------|
| Setup brief / minutes left | **`yard_setup`** |
| Status / next gate | `yard_publish`, `yard_pulse` |
| Open current step | `yard_publish_open` |
| Local verify | `yard_publish_verify`, `yard_publish_watch` |
| Confirm / next | `yard_publish_confirm`, `yard_publish_next` — Confirm only after human finished |
| Vendor URLs | `yard_portal`, `yard_guide` |
| Put secret **NAME** | `yard_env_put` `{ provider, name, spawn: true }` — **never** pass `value` |
| itch butler push | `yard_butler_push` or `orbityard butler push --target user/game:channel` — never holds itch credentials |
| Hosted deploy | Prefer Desktop Deploy; else `yard_hostdeploy` if CLI session exists |

## Never

- Paste API keys / tokens into tool args or chat (`pdl_…`, OATs, wrangler tokens)
- `yard_human put:true` under MCP (no TTY — it will fail; use `yard_env_put`)
- Claim “OAuth done” / “checkout works” without Verify or human Confirm evidence
- Pass `value` / `passphrase` to `yard_vault` in MCP — prefer Desktop export
- Grow Desktop Integrations / coach UI — out of scope
- Auto-Confirm live deploy or store submit

## Validate means

Doctor PATH, publish plan honesty, last deploy URL from host CLI, env **names** present — not CDP-filling Paddle or store review green.

Canon: `specs/backend/agent-setup-human-confirm-design.md` · `specs/backend/mcp-assist-contract-design.md` · `specs/backend/mcp-agent-guide-design.md`
