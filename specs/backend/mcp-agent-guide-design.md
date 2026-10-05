# MCP agent guide — Client thin, agents open + validate

**Status:** Design — G1–G4 + `yard_setup` + project skill shipped; Client thin  
**Updated:** 2026-10-04  
**Owner:** `orbityard mcp` (`crates/orbityard/src/mcp.rs`)  
**Parent:** [mcp-assist-contract-design](./mcp-assist-contract-design.md) · [agent-setup-human-confirm-design](./agent-setup-human-confirm-design.md) · [close-cut-design](./close-cut-design.md) · [human-gate-catalog-design](./human-gate-catalog-design.md)

## Plan alignment

- Maintainer: keep **Desktop streamlined**; put “guide to the page + help validate” on **MCP / agents**, not more Client wizards.
- **CANCELLED:** Vendor handoff coach (in-app overlay / CDP form-fill).
- **PAUSED:** Portal / Integrations UX expansion on Desktop.
- **In scope:** Agents call `yard_*` → exact `entry_url` · tell human what to do on that page · `yard_setup` / `yard_pulse` / `yard_publish` / `yard_publish_watch` for **local** validation.
- **Out of scope:** Agent creates API keys, pastes secrets, finishes OAuth, uploads Steam depots, auto-Confirms live publish.

## Why this is the remaining product

Easy vendors (Vercel, Polar) need a URL + official CLI. Hard vendors still own token-create. A second Client loses. An agent in the editor the operator already uses can:

1. `yard_setup` → phase + human action + secret **NAME** (not value).
2. `yard_portal` / `yard_guide` → **one URL** (not encyclopedia).
3. Say the field name (e.g. default payment link, Polar checkout URL).
4. After the human returns: `yard_pulse` / `yard_publish` / doctor — **validate the project**, not the vendor account.

Same kernel as CLI. No new shell.

## Client rule

Desktop stays: bind folder · Publish spine · Deploy (official CLI) · Open dashboard. Do **not** add Integrations copy, coaches, or extra Confirm chrome for this band.

## Agent loop (canon)

```text
1. yard_setup / yard_doctor / yard_pulse → phase + human action + local signals
2. yard_publish                 → required next gate (read-only)
3. yard_portal | yard_guide     → entry_url for that gate (open=true only if human wants browser)
4. Human does vendor UI / Login CLI / Put in a real TTY
5. yard_publish_watch | pulse   → local Verify; never “OAuth is done” without evidence
6. Human Confirm on CLI/Desktop for irreversible steps
```

**Validate products** here means: Signet/doctor PATH, publish plan honesty, deploy URL from last host CLI, listing URLs present in env **names** not values. It does **not** mean CDP-assert Paddle overlay or store review green.

## Tooling (existing — prefer before new tools)

| Need | Tool |
|------|------|
| Next URL | `yard_portal`, `yard_guide` (`open` optional) |
| Secret **names** + put recipe | `yard_secrets`, `yard_env`, `yard_env_put` (spawn terminal; no values) |
| Checklist | `yard_assist` |
| Local validate | `yard_pulse`, `yard_status`, `yard_publish_watch` |
| Hosted deploy | Prefer Desktop Deploy; `yard_hostdeploy` if CLI session exists |

S2.4 **G1–G4 shipped** — publish mutations · `yard_env_put` · human put TTY bail · vault docs. G5 hint already on hostdeploy; G6 CDP stays Later.

## Skills (Cursor)

Project skill: [`.cursor/skills/orbit-yard-mcp-agent/SKILL.md`](../../.cursor/skills/orbit-yard-mcp-agent/SKILL.md) — prefer `ship_*` over Desktop wizards; never hold keys.

## Acceptance

- [x] Desktop: no new wizard surfaces in this band
- [x] Design + MCP inventory agree (this file + mcp.rs `tools()`)
- [x] Impl slice: G1 wrappers
- [x] Impl slice: G2 `yard_env_put`
- [x] Impl slice: G3 human put TTY bail · G4 vault schema soften
- [x] Cursor project skill `orbit-yard-mcp-agent`
- [ ] L3: agent session uses `yard_publish_verify` then human Confirm (not required to ship G1)

## Do not

- Reintroduce in-app tooltips on vendor pages  
- `yard_env_put` that types the secret  
- Claim MCP “sets up Paddle” end-to-end
