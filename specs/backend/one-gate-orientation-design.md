# One-gate orientation — Publish is the next step

**Status:** Active — S3.3 slice 1  
**Updated:** 2026-10-04  
**Parent:** [agent-pause-resume-design](./agent-pause-resume-design.md) · close-cut  
**Surfaces:** MCP pause copy · Desktop Integrations banner  
**PAUSED:** Portal / Integrations UX expansion (this band **subtracts** confusion, does not add wizards)

```text
GOAL:     Client + MCP name the same current Publish gate. Integrations is not
          “the next step” unless that gate is listing.* / marketing / env.sprint-from-payments.
NOT:      New coaches · Paddle MCP packs · more Integrations chrome
```

## Problem

Operator logged in in the browser and opened Integrations → Paddle while MCP was paused on **Scopes**. Three UIs claimed to be next. Official vendor pages stay; Orbit Yard must not add a second school.

## Contracts

### MCP `pause`

When paused, `pause.prompt` starts with:

`Current Publish gate \`{step_id}\`: {title}. …`

Plus `pause.current_gate`, `pause.desktop_view`, `pause.integrations_ok` (true only for `listing.*`, `marketing.*`, or `env.sprint` when commerce Put is the gate).

If `integrations_ok` is false: *Stay on Publish (Targets for scopes · Deployment for host login). Integrations is not this gate.*

### Desktop

On Integrations, if current Publish step is **not** commerce: show banner  
`Current gate: {title} — go there` → Publish (or Targets if `desktop_view` is scopes).

Confirm gate / Continue publishing unchanged (already hand off to Publish).

### Agent skill

Pause prompt is the only next action. Do not send the operator to Integrations unless `integrations_ok`.

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p orbityard mcp::tests` · pause includes `current_gate` |
| L2 | Open Integrations while current is scopes → banner names Scopes |

## Do not

- Expand Paddle wizard copy  
- Auto-Confirm Scopes because someone opened a vendor login  
