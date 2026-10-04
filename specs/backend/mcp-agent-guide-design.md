# MCP agent guide — Client thin, agents open + validate

**Status:** Design (active band after close-cut)  
**Updated:** 2026-10-04  
**Owner:** `shipctl mcp` (`crates/shipctl/src/mcp.rs`)  
**Parent:** [mcp-assist-contract-design](./mcp-assist-contract-design.md) · [close-cut-design](./close-cut-design.md) · [human-gate-catalog-design](./human-gate-catalog-design.md)

## Plan alignment

- Maintainer: keep **Desktop streamlined**; put “guide to the page + help validate” on **MCP / agents**, not more Client wizards.
- **CANCELLED:** Vendor handoff coach (in-app overlay / CDP form-fill).
- **PAUSED:** Portal / Integrations UX expansion on Desktop.
- **In scope:** Agents call `ship_*` → exact `entry_url` · tell human what to do on that page · `ship_pulse` / `ship_publish` / `ship_publish_watch` for **local** validation.
- **Out of scope:** Agent creates API keys, pastes secrets, finishes OAuth, uploads Steam depots, auto-Confirms live publish.

## Why this is the remaining product

Easy vendors (Vercel, Polar) need a URL + official CLI. Hard vendors still own token-create. A second Client loses. An agent in the editor the operator already uses can:

1. `ship_portal` / `ship_guide` → **one URL** (not encyclopedia).
2. Say the field name (e.g. default payment link, Polar checkout URL).
3. After the human returns: `ship_pulse` / `ship_publish` / doctor — **validate the project**, not the vendor account.

Same kernel as CLI. No new shell.

## Client rule

Desktop stays: bind folder · Publish spine · Deploy (official CLI) · Open dashboard. Do **not** add Integrations copy, coaches, or extra Confirm chrome for this band.

## Agent loop (canon)

```text
1. ship_doctor / ship_pulse     → local tools + signals
2. ship_publish                 → required next gate (read-only)
3. ship_portal | ship_guide     → entry_url for that gate (open=true only if human wants browser)
4. Human does vendor UI / Login CLI / Put in a real TTY
5. ship_publish_watch | pulse   → local Verify; never “OAuth is done” without evidence
6. Human Confirm on CLI/Desktop for irreversible steps
```

**Validate products** here means: Signet/doctor PATH, publish plan honesty, deploy URL from last host CLI, listing URLs present in env **names** not values. It does **not** mean CDP-assert Paddle overlay or store review green.

## Tooling (existing — prefer before new tools)

| Need | Tool |
|------|------|
| Next URL | `ship_portal`, `ship_guide` (`open` optional) |
| Secret **names** + put recipe | `ship_secrets`, `ship_env` — no values |
| Checklist | `ship_assist` |
| Local validate | `ship_pulse`, `ship_status`, `ship_publish_watch` |
| Hosted deploy | Prefer Desktop Deploy; `ship_hostdeploy` if CLI session exists |

S2.4 **G1 shipped** — `ship_publish_open` / `ship_publish_verify` / `ship_publish_confirm` / `ship_publish_next`. G6 CDP stays Later.

## Skills (Cursor)

A short project skill may tell the agent: use `shipctl mcp` first; official vendor MCP (Paddle docs, etc.) second; never hold `pdl_` / OATs in chat. Skills are prompts, not a fourth Client.

## Acceptance

- [x] Desktop: no new wizard surfaces in this band
- [x] Design + MCP inventory agree (this file + mcp.rs `tools()`)
- [x] Impl slice: G1 wrappers
- [ ] L3: agent session uses `ship_publish_verify` then human Confirm (not required to ship G1)

## Do not

- Reintroduce in-app tooltips on vendor pages  
- `ship_env_put` that types the secret  
- Claim MCP “sets up Paddle” end-to-end
