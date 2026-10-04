---
name: ship-mcp-agent
description: >-
  Guide Ship Studio release work via shipctl MCP (ship_* tools): next publish
  gate, exact vendor URLs, env Put launch, local Verify. Use when shipping,
  deploying, signing, putting secrets, or validating a Ship Studio / shipctl
  project — prefer MCP over Desktop Integrations wizards.
---

# Ship Studio — MCP agent guide

Client stays thin. You assist through **`shipctl mcp`** tools. You do **not** hold secrets, finish OAuth, create vendor API keys, or upload store binaries.

## Loop

```text
1. ship_doctor / ship_pulse          → local tools + signals
2. ship_publish                      → required next gate (read-only)
3. ship_portal | ship_guide          → one entry_url (open=true only if human wants browser)
4. Human: vendor UI / Login CLI / Put in a real TTY
5. ship_publish_verify | ship_publish_watch → local Verify only
6. Human Confirm → ship_publish_confirm → ship_publish_next
```

## Tools (prefer these)

| Need | Tool |
|------|------|
| Status / next gate | `ship_publish`, `ship_pulse` |
| Open current step | `ship_publish_open` |
| Local verify | `ship_publish_verify`, `ship_publish_watch` |
| Confirm / next | `ship_publish_confirm`, `ship_publish_next` — Confirm only after human finished |
| Vendor URLs | `ship_portal`, `ship_guide` |
| Put secret **NAME** | `ship_env_put` `{ provider, name, spawn: true }` — **never** pass `value` |
| itch butler push | `ship_butler_push` or `shipctl butler push --target user/game:channel` — never holds itch credentials |
| Hosted deploy | Prefer Desktop Deploy; else `ship_hostdeploy` if CLI session exists |

## Never

- Paste API keys / tokens into tool args or chat (`pdl_…`, OATs, wrangler tokens)
- `ship_human put:true` under MCP (no TTY — it will fail; use `ship_env_put`)
- Claim “OAuth done” / “checkout works” without Verify or human Confirm evidence
- Pass `value` / `passphrase` to `ship_vault` in MCP — prefer Desktop export
- Grow Desktop Integrations / coach UI — out of scope

## Validate means

Doctor PATH, publish plan honesty, last deploy URL from host CLI, env **names** present — not CDP-filling Paddle or store review green.

Canon: `specs/backend/mcp-assist-contract-design.md` · `specs/backend/mcp-agent-guide-design.md`
