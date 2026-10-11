---
name: orbit-yard-mcp-agent
description: >-
  Guide Orbit Yard release work via orbityard MCP (yard_* tools): next publish
  gate, pause for human auth/keys, resume after confirm, env Put launch, local
  Verify. Use when shipping, deploying, signing, putting secrets, or validating
  an Orbit Yard / orbityard project — prefer MCP over Desktop Integrations wizards.
---

# Orbit Yard — MCP agent guide

Client stays thin. You assist through **`orbityard mcp`** tools. You do **not** hold secrets, finish OAuth, create vendor API keys, or upload store binaries.

## Founder Get-live spine (mandatory map)

`yard_setup.spine` names where you are on the five-step path (not Integrations):

```text
1 Open project → 2 Choose host → 3 Login once → 4 Env + Deploy → 5 Open live · Confirm
```

Prefer **Get live** tools: `yard_publish_open` · `yard_hostdeploy` · `yard_env_put` / `yard_env_import` · `yard_publish_verify` · Confirm. Soft-no: Desktop Integrations as the deploy path.

## Pause / resume (mandatory)

```text
1. yard_setup → read spine.step + pause
2. If paused=true → STOP. Show pause.prompt (spine N/5 · one Publish gate).
   Do **not** send the operator to Integrations unless `pause.integrations_ok`.
   Allowed while paused: yard_publish_open / yard_env_put (spawn) only if the pause asks.
3. Wait until the human says they finished (“done”, “resume”, “continue”).
4. yard_resume → read still_paused / need_confirm / next_tools / setup.spine
5. If need_confirm → human Confirms → yard_publish_confirm → yard_publish_next
6. Loop yard_setup until paused=false or phase=done
```

Never invent “OAuth done” or “token configured” without resume/Confirm evidence.
Do not treat Integrations / Paddle as the current gate unless `integrations_ok`.

## Minutes-to-launch loop (Get live)

```text
1. yard_setup                     → spine + phase + pause + human action
2. yard_publish_open              → browser and/or Login CLI (visible TTY)  [spine 3]
3. Human: Login / create key / Put
4. yard_env_put | yard_env_import | yard_hostdeploy   [spine 4] — NAME only; never value
5. Human: “resume”
6. yard_resume → yard_publish_verify as needed        [spine 5]
7. Human Confirm → yard_publish_confirm → yard_publish_next
```

## Tools (prefer these)

| Need | Tool |
|------|------|
| Setup / pause brief | **`yard_setup`** |
| After human finishes a gate | **`yard_resume`** |
| Status / next gate | `yard_publish`, `yard_pulse` |
| Open current step | `yard_publish_open` |
| Local verify | `yard_publish_verify`, `yard_publish_watch` |
| Confirm / next | `yard_publish_confirm`, `yard_publish_next` — Confirm only after human finished |
| Vendor URLs | `yard_portal`, `yard_guide` |
| Put secret **NAME** | `yard_env_put` `{ provider, name, spawn: true }` — **never** pass `value` |
| itch butler push | `yard_butler_push` — never holds itch credentials |
| Hosted deploy | Prefer Desktop Deploy; else `yard_hostdeploy` if CLI session exists |

## Never

- Paste API keys / tokens into tool args or chat
- Continue mutating while `paused=true` without human resume
- `yard_human put:true` under MCP (use `yard_env_put`)
- Claim “OAuth done” without Verify / resume / Confirm evidence
- Pass `value` / `passphrase` to `yard_vault` in MCP
- Grow Desktop Integrations / coach UI
- Auto-Confirm live deploy or store submit
- Paste Paddle-style multi-step prompt packs as a substitute for tools

## Validate means

Doctor PATH, publish plan honesty, last deploy URL from host CLI, env **names** present — not CDP-filling Paddle or store review green.

Canon: `specs/backend/founder-minimal-spine-design.md` (F4) · `specs/backend/agent-pause-resume-design.md` · `specs/backend/agent-setup-human-confirm-design.md` · `specs/backend/mcp-assist-contract-design.md`
