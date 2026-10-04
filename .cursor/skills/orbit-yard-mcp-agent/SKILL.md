---
name: orbit-yard-mcp-agent
description: >-
  Guide Orbit Yard release work via orbityard MCP (orbit_* tools): next publish
  gate, exact vendor URLs, env Put launch, local Verify. Use when shipping,
  deploying, signing, putting secrets, or validating an Orbit Yard / orbityard
  project — prefer MCP over Desktop Integrations wizards.
---

# Orbit Yard — MCP agent guide

Client stays thin. You assist through **`orbityard mcp`** tools. You do **not** hold secrets, finish OAuth, create vendor API keys, or upload store binaries.

## Loop

```text
1. orbit_doctor / orbit_pulse          → local tools + signals
2. orbit_publish                      → required next gate (read-only)
3. orbit_portal | orbit_guide          → one entry_url (open=true only if human wants browser)
4. Human: vendor UI / Login CLI / Put in a real TTY
5. orbit_publish_verify | orbit_publish_watch → local Verify only
6. Human Confirm → orbit_publish_confirm → orbit_publish_next
```

## Tools (prefer these)

| Need | Tool |
|------|------|
| Status / next gate | `orbit_publish`, `orbit_pulse` |
| Open current step | `orbit_publish_open` |
| Local verify | `orbit_publish_verify`, `orbit_publish_watch` |
| Confirm / next | `orbit_publish_confirm`, `orbit_publish_next` — Confirm only after human finished |
| Vendor URLs | `orbit_portal`, `orbit_guide` |
| Put secret **NAME** | `orbit_env_put` `{ provider, name, spawn: true }` — **never** pass `value` |
| itch butler push | `orbit_butler_push` or `orbityard butler push --target user/game:channel` — never holds itch credentials |
| Hosted deploy | Prefer Desktop Deploy; else `orbit_hostdeploy` if CLI session exists |

## Never

- Paste API keys / tokens into tool args or chat (`pdl_…`, OATs, wrangler tokens)
- `orbit_human put:true` under MCP (no TTY — it will fail; use `orbit_env_put`)
- Claim “OAuth done” / “checkout works” without Verify or human Confirm evidence
- Pass `value` / `passphrase` to `orbit_vault` in MCP — prefer Desktop export
- Grow Desktop Integrations / coach UI — out of scope

## Validate means

Doctor PATH, publish plan honesty, last deploy URL from host CLI, env **names** present — not CDP-filling Paddle or store review green.

Canon: `specs/backend/mcp-assist-contract-design.md` · `specs/backend/mcp-agent-guide-design.md`
