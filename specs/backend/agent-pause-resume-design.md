# Agent pause / resume — human gates

**Status:** Active — S3.2 implement  
**Updated:** 2026-10-04  
**Parent:** [agent-setup-human-confirm-design](./agent-setup-human-confirm-design.md) · [mcp-assist-contract-design](./mcp-assist-contract-design.md)  
**Surfaces:** MCP `yard_*` only (no Desktop Integrations growth)  
**PAUSED:** Portal / Integrations UX · Vendor coach (**CANCELLED**)

```text
GOAL:     At auth / token create / Put / Confirm gates, agents STOP and prompt the
          human. After the human says they finished, agents call yard_resume and continue.
NOT:      Agent creates keys · pastes secret values · auto-Confirms live · CDP · prompt packs
```

## Protocol

```text
Agent                         Human                      Kernel
─────                         ─────                      ──────
yard_setup                    sees pause.prompt
  paused=true              →  create key / Login CLI / Put
  STOP chatting tools         says “done” / “resume”
yard_resume                ←  (optional Confirm on Client)
  verify + need_confirm?
yard_publish_confirm       ←  human attests irreversible
yard_publish_next          →  next gate / next pause
```

### Agent rules (mandatory)

1. If `paused == true` → **do not** call further mutate tools in the same turn except Open / Put spawn that the pause itself requested.  
2. Tell the operator `pause.prompt` (title + detail + entry_url / secret NAME).  
3. Wait for explicit resume language (“done”, “resume”, “I finished”, “continue”).  
4. Call **`yard_resume`** — never invent “OAuth done.”  
5. If `need_confirm` → ask human to Confirm, then `yard_publish_confirm` → `yard_publish_next`.  
6. Loop `yard_setup` until `paused == false` and phase is `done` or next Auto tools run.

## Contracts

### `yard_setup` (extended)

Adds:

```json
{
  "paused": true,
  "pause": {
    "schema": "orbit-yard/pause/v1",
    "reason": "human_gate",
    "wait_for": "create_key|login_cli|put|confirm|open_dashboard",
    "prompt": "Create Netlify token on … then say resume.",
    "resume_tool": "yard_resume"
  }
}
```

`paused` is true iff `human.action != "none"` and phase is not `done`.

### `yard_resume`

Read + local Verify only (no Confirm unless already Done). Returns:

```json
{
  "schema": "orbit-yard/resume/v1",
  "still_paused": true,
  "verified": false,
  "verify_message": "…",
  "need_confirm": true,
  "message": "Human finished vendor UI — Confirm this gate, then Next.",
  "setup": { "...SetupBrief..." },
  "next_tools": ["yard_publish_confirm", "yard_publish_next"]
}
```

| Field | Meaning |
|-------|---------|
| `still_paused` | Current gate still needs human (or Confirm) |
| `need_confirm` | Local Verify ok / human_attest — call Confirm after human attests |
| `verified` | Result of `verify_current` this call |

Never auto-Confirms Deploy / store / live publish.

### `yard_pause` (optional alias)

Same payload as `yard_setup` focused on the pause block — thin wrapper for agents that look for “pause.”

## Client

No new Desktop chrome. Publish Open / Confirm already match this spine.

## Acceptance

| Layer | Proof |
|-------|--------|
| L1 | `cargo test -p orbityard mcp::tests` · pause/resume unit |
| L2 | Skill documents STOP → resume → Confirm |
| L3 | Manual: agent pauses at oauth/env · human resumes · next |

## Do not

- Reopen Integrations / coach  
- `yard_resume` that Confirms without human  
- Treat Paddle MCP prompt packs as the pattern  
