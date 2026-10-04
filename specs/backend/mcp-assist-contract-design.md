# MCP assist contract — design

**Status:** O3 done — contract + gap list for S2.4; H5 hosted playbook + `orbit_hostdeploy`  
**Updated:** 2026-09-27  
**Parent:** [ship-studio-overhaul-design.md](./ship-studio-overhaul-design.md)  
**Surfaces:** [surfaces-cli-tui-desktop.md](./surfaces-cli-tui-desktop.md) · `orbityard mcp` (`crates/orbityard/src/mcp.rs`)  
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
| **orbityard** | Shared kernel under Client and MCP |

## Invariants

1. **No secret custody** — `orbit_secrets` / `orbit_env` / `orbit_portal` return names and URLs only, never secret values from disk.  
2. **Put is human-TTY** — Prefer Desktop **Put** or `orbityard human --put` / `env --put` in a terminal the operator can see. Agents must not claim to paste secrets for the user.  
3. **`orbit_human` `put: true`** — Interactive; only when the operator has a TTY attached to the MCP host process. Default agent advice: open Desktop Put or a terminal, don’t run put headlessly.  
4. **`orbit_vault`** — Optional encrypted export; if `value` is passed in tool args, agents must not echo it into chat/logs. Prefer Client Put + vault export from Desktop.  
5. **Publish mutations** — `orbit_publish` is status; `orbit_publish_open` / `orbit_publish_verify` / `orbit_publish_confirm` / `orbit_publish_next` mirror CLI. Confirm is human attest; never live npm/cargo/`gh release create` / docker push / store upload.  
6. **`orbit_publish_watch` `auto_confirm`** — Only confirms when local Verify ok; never live npm/cargo/`gh release create` / docker push.  
7. **`orbit_deploy` / `orbit_flow` / `orbit_hostdeploy`** — Network; may prompt. Prefer Desktop Deployment **Deploy** (or Advanced N-class terminal). Headless MCP may soft-fail on auth — tell human Login CLI / Sign in (web). `orbit_deploy` = Orbit only; hosted Tier A = `orbit_hostdeploy`.  
8. **Docs URLs** — Agents may surface `docs_url` as Learn more; primary next act follows [human-gate catalog](./human-gate-catalog-design.md) (Put / Login / Confirm).

## Tool inventory (audit 2026-09-25)

| Tool | Class | Safe for agents | Notes |
|------|-------|-----------------|-------|
| `orbit_doctor` | J | Yes | Offline tools check |
| `orbit_configure` | J | Yes | Writes studio.json intent |
| `orbit_portal` | J (+ open) | Yes | Plan + optional open entry URLs |
| `orbit_secrets` | J (+ open) | Yes | Names + put CLI; no values |
| `orbit_env` | J | Yes | Plan only |
| `orbit_env_put` | J (+ spawn) | Yes | NAME only; recipe + optional external TTY; rejects `value` |
| `orbit_butler_push` | J (+ spawn) | Yes | itch target + dir; recipe / terminal; no credentials |
| `orbit_scopes` | J | Yes | Detect scopes |
| `orbit_sign_paths` | J | Yes | Self vs official |
| `orbit_assist` | J | Yes | Checklist |
| `orbit_guide` / `orbit_ship` | J (+ open) | Yes | Offline prep; open URLs optional |
| `orbit_pulse` / `orbit_status` | J | Yes | Local signals |
| `orbit_launch` | J | Yes (read) | Status only — mutations stay CLI/Desktop |
| `orbit_publish` | J | Yes (read) | Status only |
| `orbit_publish_open` | J / N | Caution | Opens URL / may run login CLI — human TTY |
| `orbit_publish_verify` | J | Yes | Local Verify JSON `{ok,message,publish}` — not vendor HTTPS |
| `orbit_publish_confirm` | J | Caution | Human attest only — agent must not invent Confirm |
| `orbit_publish_next` | J | Yes | Advance plan; `force` optional |
| `orbit_publish_watch` | J | Yes | Local Verify poll; optional auto_confirm |
| `orbit_flow_dry_run` | J | Yes | Plan print |
| `orbit_flow` / `orbit_sign` / `orbit_deploy` | N / J | Caution | Deploy/sign may need TTY; deploy refuses offline; Orbit only |
| `orbit_hostdeploy` | N | Caution | Hosted CLI (CF/Vercel/Netlify); prefer Desktop Deploy; no secret custody |
| `orbit_human` | T when put | Caution | MCP: `put:true` **bails** without TTY — use `orbit_env_put` |
| `orbit_vault` | L+custody risk | Caution | Schema discourages value/passphrase in MCP args |

## Agent playbook (human gates)

```text
1. orbit_pulse / orbit_publish → what’s required next
2. If secret_put → `orbit_env_put` { provider, name, spawn:true } or Desktop Put — never paste values into tool args
3. If oauth_login → tell human: Desktop Login CLI / Sign in (web) / orbityard portal
4. Open dashboard URLs via orbit_portal open=true only as secondary
5. After human finishes → `orbit_publish_verify` or `orbit_publish_watch` → human-ok then `orbit_publish_confirm` → `orbit_publish_next`
6. Never say “I stored your API token” or “OAuth is done” without Verify/Confirm evidence
```

## Agent playbook (hosted deploy — Path B)

See [hosted-deploy-ops-value-bar H5](./hosted-deploy-ops-value-bar-design.md). Short form:

```text
1. orbit_portal → Sign in (web) / Login CLI (human)
2. Prefer Desktop Deployment Deploy for TTY auth
3. orbit_hostdeploy { project, provider } when CLI session exists
4. orbit_pulse / orbit_status → hosted urls; classify [auth] → human Login CLI
5. Never Create Token form-fill; never paste secrets into tool args
6. Publish Live check remains human Confirm
```

## Gaps (S2.4 backlog)

| Gap | Why it hurts | Suggested follow-up |
|-----|--------------|---------------------|
| **G1** `orbit_publish_open` / `confirm` / `next` / `verify` | **Shipped** — MCP wrappers | Agents still must not fake Confirm |
| **G2** `orbit_env_put` recipe + optional terminal spawn | **Shipped** — never accepts `value` | Prefer `spawn:true` for a visible TTY |
| **G3** `orbit_human` put without TTY | **Shipped** — MCP bails; prefer `orbit_env_put` | CLI `--put` still soft-skips |
| **G4** `orbit_vault` value-in-args | **Shipped** — schema/docs discourage value/passphrase in MCP | Prefer Desktop export |
| **G5** Deploy/flow/hostdeploy from MCP vs Desktop | Auth prompts headless | Prefer Desktop Deploy; `orbit_hostdeploy` hint on fail (**H5 shipped**) |
| **G6** CDP / live Desktop MCP dogfood | Can’t prove agent↔Desktop loop | Reliability Later / CDP attach |

## Proof (O3)

- This design + inventory matches `crates/orbityard/src/mcp.rs` `tools()`  
- S2.4 backlog points here  
- No claim that MCP finishes OAuth or holds keys  

## Activation

Overhaul **O3 closed** → handoff **O4 Catalog diet**.
