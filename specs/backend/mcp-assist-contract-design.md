# MCP assist contract — design

**Status:** O3 done — contract + gap list for S2.4; H5 hosted playbook + `ship_hostdeploy`  
**Updated:** 2026-09-27  
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
5. **Publish mutations** — `ship_publish` is status; `ship_publish_open` / `ship_publish_verify` / `ship_publish_confirm` / `ship_publish_next` mirror CLI. Confirm is human attest; never live npm/cargo/`gh release create` / docker push / store upload.  
6. **`ship_publish_watch` `auto_confirm`** — Only confirms when local Verify ok; never live npm/cargo/`gh release create` / docker push.  
7. **`ship_deploy` / `ship_flow` / `ship_hostdeploy`** — Network; may prompt. Prefer Desktop Deployment **Deploy** (or Advanced N-class terminal). Headless MCP may soft-fail on auth — tell human Login CLI / Sign in (web). `ship_deploy` = Orbit only; hosted Tier A = `ship_hostdeploy`.  
8. **Docs URLs** — Agents may surface `docs_url` as Learn more; primary next act follows [human-gate catalog](./human-gate-catalog-design.md) (Put / Login / Confirm).

## Tool inventory (audit 2026-09-25)

| Tool | Class | Safe for agents | Notes |
|------|-------|-----------------|-------|
| `ship_doctor` | J | Yes | Offline tools check |
| `ship_configure` | J | Yes | Writes studio.json intent |
| `ship_portal` | J (+ open) | Yes | Plan + optional open entry URLs |
| `ship_secrets` | J (+ open) | Yes | Names + put CLI; no values |
| `ship_env` | J | Yes | Plan only |
| `ship_env_put` | J (+ spawn) | Yes | NAME only; recipe + optional external TTY; rejects `value` |
| `ship_scopes` | J | Yes | Detect scopes |
| `ship_sign_paths` | J | Yes | Self vs official |
| `ship_assist` | J | Yes | Checklist |
| `ship_guide` / `ship_ship` | J (+ open) | Yes | Offline prep; open URLs optional |
| `ship_pulse` / `ship_status` | J | Yes | Local signals |
| `ship_launch` | J | Yes (read) | Status only — mutations stay CLI/Desktop |
| `ship_publish` | J | Yes (read) | Status only |
| `ship_publish_open` | J / N | Caution | Opens URL / may run login CLI — human TTY |
| `ship_publish_verify` | J | Yes | Local Verify JSON `{ok,message,publish}` — not vendor HTTPS |
| `ship_publish_confirm` | J | Caution | Human attest only — agent must not invent Confirm |
| `ship_publish_next` | J | Yes | Advance plan; `force` optional |
| `ship_publish_watch` | J | Yes | Local Verify poll; optional auto_confirm |
| `ship_flow_dry_run` | J | Yes | Plan print |
| `ship_flow` / `ship_sign` / `ship_deploy` | N / J | Caution | Deploy/sign may need TTY; deploy refuses offline; Orbit only |
| `ship_hostdeploy` | N | Caution | Hosted CLI (CF/Vercel/Netlify); prefer Desktop Deploy; no secret custody |
| `ship_human` | T when put | Caution | MCP: `put:true` **bails** without TTY — use `ship_env_put` |
| `ship_vault` | L+custody risk | Caution | Schema discourages value/passphrase in MCP args |

## Agent playbook (human gates)

```text
1. ship_pulse / ship_publish → what’s required next
2. If secret_put → `ship_env_put` { provider, name, spawn:true } or Desktop Put — never paste values into tool args
3. If oauth_login → tell human: Desktop Login CLI / Sign in (web) / shipctl portal
4. Open dashboard URLs via ship_portal open=true only as secondary
5. After human finishes → `ship_publish_verify` or `ship_publish_watch` → human-ok then `ship_publish_confirm` → `ship_publish_next`
6. Never say “I stored your API token” or “OAuth is done” without Verify/Confirm evidence
```

## Agent playbook (hosted deploy — Path B)

See [hosted-deploy-ops-value-bar H5](./hosted-deploy-ops-value-bar-design.md). Short form:

```text
1. ship_portal → Sign in (web) / Login CLI (human)
2. Prefer Desktop Deployment Deploy for TTY auth
3. ship_hostdeploy { project, provider } when CLI session exists
4. ship_pulse / ship_status → hosted urls; classify [auth] → human Login CLI
5. Never Create Token form-fill; never paste secrets into tool args
6. Publish Live check remains human Confirm
```

## Gaps (S2.4 backlog)

| Gap | Why it hurts | Suggested follow-up |
|-----|--------------|---------------------|
| **G1** `ship_publish_open` / `confirm` / `next` / `verify` | **Shipped** — MCP wrappers | Agents still must not fake Confirm |
| **G2** `ship_env_put` recipe + optional terminal spawn | **Shipped** — never accepts `value` | Prefer `spawn:true` for a visible TTY |
| **G3** `ship_human` put without TTY | **Shipped** — MCP bails; prefer `ship_env_put` | CLI `--put` still soft-skips |
| **G4** `ship_vault` value-in-args | **Shipped** — schema/docs discourage value/passphrase in MCP | Prefer Desktop export |
| **G5** Deploy/flow/hostdeploy from MCP vs Desktop | Auth prompts headless | Prefer Desktop Deploy; `ship_hostdeploy` hint on fail (**H5 shipped**) |
| **G6** CDP / live Desktop MCP dogfood | Can’t prove agent↔Desktop loop | Reliability Later / CDP attach |

## Proof (O3)

- This design + inventory matches `crates/shipctl/src/mcp.rs` `tools()`  
- S2.4 backlog points here  
- No claim that MCP finishes OAuth or holds keys  

## Activation

Overhaul **O3 closed** → handoff **O4 Catalog diet**.
