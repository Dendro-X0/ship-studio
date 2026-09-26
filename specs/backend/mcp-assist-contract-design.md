# MCP assist contract — design

**Status:** O3 done — contract + gap list for S2.4  
**Updated:** 2026-09-25  
**Parent:** [ship-studio-overhaul-design.md](./ship-studio-overhaul-design.md)  
**Surfaces:** [surfaces-cli-tui-desktop.md](./surfaces-cli-tui-desktop.md) · `shipctl mcp` (`crates/shipctl/src/mcp.rs`)  
**Gates:** [human-gate-catalog-design.md](./human-gate-catalog-design.md)

```text
GOAL:     Agents assist release via ship_* without holding keys or finishing
          OAuth/store/DNS. Same spine as Client; human remains authority.
NOT:      “AI finishes publish” · MCP as password manager · auto live publish
```

## Role (same pattern as CodaCtrl / Ghidra MCP)

| Actor | Does |
|-------|------|
| **Agent** | Call `ship_*` to detect, plan, open URLs, dry-run, watch Verify, explain next gate |
| **Human** | Login CLI · Put secrets in a real TTY · Confirm irreversible · vendor UI |
| **shipctl** | Shared kernel under Client and MCP |

## Invariants

1. **No secret custody** — `ship_secrets` / `ship_env` / `ship_portal` return names and URLs only, never secret values from disk.  
2. **Put is human-TTY** — Prefer Desktop **Put** or `shipctl human --put` / `env --put` in a terminal the operator can see. Agents must not claim to paste secrets for the user.  
3. **`ship_human` `put: true`** — Interactive; only when the operator has a TTY attached to the MCP host process. Default agent advice: open Desktop Put or a terminal, don’t run put headlessly.  
4. **`ship_vault`** — Optional encrypted export; if `value` is passed in tool args, agents must not echo it into chat/logs. Prefer Client Put + vault export from Desktop.  
5. **Publish mutations** — `ship_publish` / `ship_launch` are **status** today; open/verify/confirm/next remain CLI or Desktop (gap below).  
6. **`ship_publish_watch` `auto_confirm`** — Only confirms when local Verify ok; never live npm/cargo/`gh release create` / docker push.  
7. **`ship_deploy` / `ship_flow`** — Network; may prompt. Prefer Advanced Desktop N-class terminal, or warn operator that headless MCP may soft-fail on auth.  
8. **Docs URLs** — Agents may surface `docs_url` as Learn more; primary next act follows [human-gate catalog](./human-gate-catalog-design.md) (Put / Login / Confirm).

## Tool inventory (audit 2026-09-25)

| Tool | Class | Safe for agents | Notes |
|------|-------|-----------------|-------|
| `ship_doctor` | J | Yes | Offline tools check |
| `ship_configure` | J | Yes | Writes studio.json intent |
| `ship_portal` | J (+ open) | Yes | Plan + optional open entry URLs |
| `ship_secrets` | J (+ open) | Yes | Names + put CLI; no values |
| `ship_env` | J | Yes | Plan only |
| `ship_scopes` | J | Yes | Detect scopes |
| `ship_sign_paths` | J | Yes | Self vs official |
| `ship_assist` | J | Yes | Checklist |
| `ship_guide` / `ship_ship` | J (+ open) | Yes | Offline prep; open URLs optional |
| `ship_pulse` / `ship_status` | J | Yes | Local signals |
| `ship_publish` / `ship_launch` | J | Yes (read) | Status only — mutations via CLI/Desktop |
| `ship_publish_watch` | J | Yes | Local Verify poll; optional auto_confirm |
| `ship_flow_dry_run` | J | Yes | Plan print |
| `ship_flow` / `ship_sign` / `ship_deploy` | N / J | Caution | Deploy/sign may need TTY; deploy refuses offline |
| `ship_human` | T when put | Caution | `put` needs interactive TTY |
| `ship_vault` | L+custody risk | Caution | Passphrase/values — don’t log |

## Agent playbook (human gates)

```text
1. ship_pulse / ship_publish → what’s required next
2. If secret_put → tell human: Desktop Portal Put (or terminal shipctl env --put)
3. If oauth_login → tell human: Desktop Login CLI / shipctl portal --login
4. Open dashboard URLs via ship_portal open=true only as secondary
5. After human finishes → ship_publish_watch (or CLI verify) → Confirm on Desktop/CLI
6. Never say “I stored your API token” or “OAuth is done” without Verify/Confirm evidence
```

## Gaps (S2.4 backlog)

| Gap | Why it hurts | Suggested follow-up |
|-----|--------------|---------------------|
| **G1** No `ship_publish_open` / `confirm` / `next` / `verify` MCP tools | Agents must shell out to CLI for minute-wizard mutations | Add thin MCP wrappers mirroring CLI subcommands |
| **G2** No `ship_env_put` that only *launches* put (non-interactive spawn recipe) | Agents can’t start Desktop/TTY Put without telling the human | Document + optional “spawn terminal” meta (platform-specific) — not headless stdin |
| **G3** `ship_human` put in MCP sessions without TTY | Silent hang / fail | Tool description + bail when non-TTY |
| **G4** `ship_vault` value-in-args | Temptation to paste secrets into agent context | Soften schema docs; prefer Desktop export |
| **G5** Deploy/flow from MCP vs Desktop N-class | Auth prompts headless | Tool descriptions: prefer Desktop Deploy terminal when Advanced |
| **G6** CDP / live Desktop MCP dogfood | Can’t prove agent↔Desktop loop | Reliability Later / CDP attach |

## Proof (O3)

- This design + inventory matches `crates/shipctl/src/mcp.rs` `tools()`  
- S2.4 backlog points here  
- No claim that MCP finishes OAuth or holds keys  

## Activation

Overhaul **O3 closed** → handoff **O4 Catalog diet**.
