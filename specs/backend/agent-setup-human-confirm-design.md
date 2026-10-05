# Agent setup + human key confirm — design

**Status:** Active — S3.1 `yard_setup` shipped (MCP brief + skill)  
**Updated:** 2026-10-04  
**Parent:** [mcp-assist-contract-design](./mcp-assist-contract-design.md) · [mcp-agent-guide-design](./mcp-agent-guide-design.md) · [human-gate-catalog-design](./human-gate-catalog-design.md)  
**Surfaces:** MCP (`yard_*`) · Client Publish spine only (no Integrations growth)  
**PAUSED:** Portal / Integrations UX expansion · Vendor coach (**CANCELLED**)

```text
GOAL:     Agents drive setup (detect, open exact vendor page, prepare Put).
          Humans create keys / Login CLI / Confirm. Agents resume after attest.
          First Public web cut feels like a few minutes, not a vendor encyclopedia.
NOT:      Agent creates API keys · pastes secrets · auto-Confirms live · CDP coaches
```

## Workflow (minutes-to-launch)

```text
Agent                          Human                         Kernel
─────                          ─────                         ──────
yard_setup / yard_publish  →   sees next gate clearly
yard_publish_open          →   browser or Login CLI opens
                               creates key on vendor UI
yard_env_put spawn:true    →   pastes in visible TTY
yard_publish_verify        →   (optional) checks local evidence
                               Confirms on Client / CLI
yard_publish_confirm       →   (human only for irreversible)
yard_publish_next          →   agent continues Auto / next Open
```

**Security split**

| Who | Keys |
|-----|------|
| Agent | Names, entry URLs, Put recipe, Verify results — **never values** |
| Human | Create token on vendor · paste into host CLI · Confirm live |
| Kernel | Sequences; stores hints/names only |

## Client (pleasant, thin)

Keep Desktop: bind · Public · Publish · Deploy.  
**Do not** add Integrations coaches (PAUSED).  

UX polish that *is* allowed in this band:

- Toast **Switch to Public** before hosted Deploy (shipped)
- Targets → Deploy aims at app folder (shipped)
- Publish shows current gate + Open / Confirm only

## MCP

### Existing tools (prefer)

| Need | Tool |
|------|------|
| Next gate | `yard_publish`, `yard_pulse` |
| Setup briefing | **`yard_setup`** (shipped) |
| Open vendor / login | `yard_publish_open`, `yard_portal` |
| Put NAME | `yard_env_put` `{ spawn: true }` |
| Local validate | `yard_publish_verify`, `yard_publish_watch` |
| Advance | `yard_publish_confirm` (human attest) · `yard_publish_next` |

### `yard_setup` contract

Read-only. Returns one JSON brief for agents (`orbit-yard/setup/v1`):

- `phase` — configure | auth | secret_put | deploy | verify | confirm | done
- `human.action` — none | login_cli | create_key | put | confirm | open_dashboard
- `human.entry_url` / `secret_name` / `provider` — never values
- `agent.next_tools` + `agent.never`
- `minutes_remaining` + publish step ref

Derivation: current Publish step + first public Put hint. No new Desktop surface.

## Acceptance

| Layer | Proof |
|-------|--------|
| L1 | `cargo test -p orbityard mcp::tests` · `setup_brief_mcp_returns_v1` |
| L2 | Skill documents minutes loop with `yard_setup` |
| L3 | Agent session: setup → human Put → verify → confirm → next (manual) |

## Do not

- Reopen Integrations / coach / CDP  
- `yard_env_put` that types the secret  
- Auto-Confirm live deploy or store submit  
- Claim “setup complete” without human Confirm when the gate is Human  
