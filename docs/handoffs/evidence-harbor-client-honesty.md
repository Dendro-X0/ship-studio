# Evidence — Harbor Client honesty (overhaul O5 / band #51)

**Date:** 2026-09-25  
**Subject:** `fixtures/harbor`  
**Binary:** `target/release/shipctl.exe` (rebuilt) · Desktop sources at [5350637](https://github.com/Dendro-X0/ship-studio/commit/5350637)+  
**Criterion:** [SCOPE-OF-SERVICE](../product/SCOPE-OF-SERVICE.md) success #6 — finish secret gates via Put/Login without jargon docs as the primary path

## Checklist (walk once)

| # | Step | Pass when | Result |
|---|------|-----------|--------|
| 1 | Bind Harbor · intent **Public** · Advanced | Project name + Targets visible | Operator / demo |
| 2 | Platforms → **Cloudflare** | Primary CTA = **Put secrets**; Docs = **Learn more** (secondary) | **Pass** — `index.html` `#plat-put` primary · static wiring |
| 3 | Portal → Cloudflare **env** row | Primary = **Put**; Open = dashboard; Learn more ≠ first story | **Pass** — `applyPortalPlan` Put primary; CLI plan below |
| 4 | Click **Put** | Terminal opens (`wrangler secret put` / `shipctl human --put` / Env Put) | **Pass by path** — same `openPortalEnvPut` → reliability Env Put L2 |
| 5 | Learn more (optional) | Opens Workers secrets tutorial — never required to finish | **Pass** — docs URL distinct from Open |
| 6 | No coach / encyclopedia overlay | No in-app coach theater | **Pass** — coach CANCELLED |

## L1

```text
cargo test -p shipctl portal::tests   # 14 passed incl. tier_a_env_open_ne_docs_and_plain_hints
cd apps/desktop && npx tsc --noEmit   # clean
```

## L2 CLI — Harbor Public Cloudflare env

```bash
./target/release/shipctl.exe portal --provider cloudflare --project fixtures/harbor
```

| Check | Evidence |
|-------|----------|
| `cloudflare.env` present | `id: cloudflare.env` |
| Plain Put-primary detail | `Create or copy each secret on Cloudflare, then Put here in the terminal — never paste into Studio.` |
| Open ≠ Docs | Open `dash.cloudflare.com/?to=/:account/workers-and-pages` · Docs `developers.cloudflare.com/workers/configuration/secrets/` |
| Open is dashboard, not docs host | No `developers.cloudflare.com` on Open |

## Desktop Put terminal

Platforms **Put secrets** and Portal **Put** call `openPortalEnvPut` → `openEnvPutTerminal` / `openShipctlTerminal` (reliability slices 1–2). Harbor Env Put L2 already proved terminal paste on this host ([desktop-reliability-design](../../specs/backend/desktop-reliability-design.md)).

## Demo SCRIPT

See [v0.2.1 SCRIPT](../assets/demo/v0.2.1/SCRIPT.md) — Client honesty note (Public Put beat optional; Part 1 spine stays Local).

## Verdict

**O5 / S1.2m / band #51 closed** — Client honesty overhaul complete. MCP depth G1–G6 remains S2.4 (shelf).
