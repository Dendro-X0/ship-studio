# Hosted deploy ops — $29 value bar (design)

**Status:** H1–H6 shipped ($29 bar + confirm/isolate)  
**Updated:** 2026-09-27  
**Parents:** [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · [studio-selfhost-guide-design](../frontend/studio-selfhost-guide-design.md) · [hosting-oauth-web-entry-design](./hosting-oauth-web-entry-design.md) · [mcp-assist-contract-design](./mcp-assist-contract-design.md)  
**Trigger:** Portal-only Open/Docs is not worth $29 vs agents that operate vendor UIs. Operators want **automation with rigor**: see the process, see the deployed project, open the provider dashboard on the result, and get help when it fails — without Studio holding secrets or filling “Create Custom Token” for them.

```text
GOAL:  Hosted Tier A (Cloudflare · Vercel · Netlify) feels like Self-host’s cousin:
       Studio streams a local vendor CLI deploy, shows phases + last-run evidence,
       deep-links dashboard + live URL, and classifies failures into next actions.
NOT:   CDP form-fill on Create Token · Studio-held API tokens · one-click without
       Login CLI when creds missing · fake “Already deployed” · coach theater
PRICE: $29 paid delta = finishable host cut for account-holders without DevOps fluency
```

## Competitive honesty

| Competitor class | They do | We do **not** |
|------------------|---------|----------------|
| Agent + browser | Click Account / Workers / TTL on Create Token | No CDP (Reliability Later); no secret custody |
| SaaS control plane | Deploy from their cloud | No Dendro datacenter |

| We earn $29 when | Mechanism |
|------------------|-----------|
| Process is visible | Streamed local `wrangler` / `vercel` / `netlify` (like `shipctl selfhost`) |
| Result is visible | Hosted URL + pulse `last_run_ok` / Orbit summary (not loopback) |
| Dashboard shows the project | Open live URL + Open dashboard (account/project deep link) |
| Failure is actionable | Troubleshoot taxonomy → Login CLI / Sign in (web) / Put / Open dashboard / retry Deploy |

Filling Cloudflare’s permission matrix for the user is **out**. Guiding them past it (prefer Sign in (web) + dashboard deploy **or** Login CLI + streamed CLI deploy) is **in**.

## Two honest hosted paths (operator chooses)

| Path | Studio role | Done signal |
|------|-------------|-------------|
| **A — Dashboard deploy** | Sign in (web) → Open dashboard → operator deploys on vendor UI → paste/copy live URL → Publish Live check → Confirm | Human Confirm |
| **B — Local CLI deploy** | Login CLI (once) → **Deploy** streams vendor CLI → last-run URLs → Open live / Open dashboard → Live check → Confirm | Stream + disk evidence; Confirm still for Publish Live check |

Both stay rigorous: Studio never stores the token value; Live check still attests.

## Target UX (Deployment detail)

For Cloudflare / Vercel / Netlify (Public intent):

| Control | Role |
|---------|------|
| **Deploy** | Primary when CLI path available — stream vendor CLI (H1) |
| **Open dashboard** | Always — see/create project on vendor (Path A) |
| **Sign in (web)** / **Portal steps** | Browser session |
| **Login CLI** | Local CLI credentials when Deploy needs them |
| **Put secrets** | Env only |
| **Results bay** | After run: phases · live URL(s) · Open live · Open dashboard · Troubleshoot |

Copy must never say Studio deploys SaaS *for* you without the local CLI or the dashboard — it **runs your CLI** or **opens their UI**.

## Slices

| Slice | Change | Proof |
|-------|--------|-------|
| **H1 — CLI Deploy stream (CF)** | Cloudflare **Deploy** → `shipctl hostdeploy --provider cloudflare`; Pages/Workers detect; streamed wrangler; hosted last-run; auth-fail → Login CLI / Sign in (web) | **Shipped** — L1 hostdeploy tests |
| **H2 — Vercel (+ Netlify)** | Same shape for `vercel --prod --yes` / `netlify deploy --prod` | **Shipped** — L1 detect tests |
| **H3 — Results bay** | Deployment aside shows last deploy evidence (phases · URL · Open live · Open dashboard) from pulse + last-run | **Shipped** — Desktop `plat-results` |
| **H4 — Troubleshoot taxonomy** | `HostFailClass` + Desktop one primary recovery (Login CLI / Preview / Retry / Open dashboard) on toast + Results bay | **Shipped** — L1 classify tests |
| **H5 — MCP playbook** | Document `ship_*` sequence for agents (no secret paste); thin `ship_hostdeploy` | **Shipped** — playbook + MCP tool L1 |
| **H6 — Confirm + isolate Results** | Per-host Results bay; Deploy confirm dialog + `--name` | **Shipped** — [host-deploy-confirm-design](../frontend/host-deploy-confirm-design.md) |

## Rigor invariants (non-negotiable)

1. Bridge never calls vendor HTTPS with secrets.  
2. No CDP / form automation on Create Custom Token (parked with Reliability Later).  
3. Prefer Sign in (web) + dashboard **or** Login CLI + streamed CLI — never imply Studio created the token.  
4. Self-host loopback ≠ hosted live ([selfhost-vs-hosted](./selfhost-vs-hosted-deploy-probe-investigation.md)).  
5. Publish Live check remains human Confirm.  
6. Dangerous automation stays rejected (registry push, live npm/cargo, store upload).

## H1 detail (next atomic)

### shipctl

- Extend or add a hosted deploy entry (prefer reusing Ritual/`deploy` path if present; else thin `shipctl` wrapper that shells to wrangler with streamed phases).  
- Detect Pages vs Workers from project layout (Harbor website → Pages).  
- Last-run: `ok`, `urls` (https only), steps `host.detect` · `host.deploy` · `host.url`, message without secrets.  
- Exit non-zero on auth → stderr classifiable.

### Desktop

- Cloudflare `openLabel` / primary: **Deploy** (stream) like Self-host; keep **Open dashboard** as sibling.  
- On auth failure toast: **Login CLI** + **Sign in (web)**.  
- Refresh pulse after run so Deploy probe shows hosted Ready only for real URLs.

### Non-goals for H1

- Creating API tokens  
- Orbit required  
- Multi-host matrix beyond Cloudflare  
- Auto-Confirm Live check  

## Proof plan (H1)

| Layer | Check |
|-------|--------|
| L1 | Unit: last-run from fixture stdout parser extracts https URL; loopback rejected |
| L2 | Harbor Public · Cloudflare · Deploy with wrangler session → Output stream + pulse hosted URL |
| L2 fail | Logged-out wrangler → troubleshoot points at Login CLI |

## H4 detail (shipped)

| Class | Primary recovery |
|-------|------------------|
| `auth` | Login CLI |
| `missing_cli` | Preview log (+ install hint) |
| `build` | Preview log |
| `network` | Retry Deploy |
| `account` / `detect` | Open dashboard (+ **Retry Deploy** after create/select project) |
| `unknown` | Preview log (+ Open dashboard / Retry) |

Missing Pages project: **Deploy auto-runs** `wrangler pages project create <name> --production-branch=main` then retries deploy once (no dashboard trip). Create failure / wrong account still → **Open dashboard** + **Retry**.

shipctl annotates failed last-run messages with `[class]`. Desktop mirrors classification for toast + Results bay **Troubleshoot** + **Retry Deploy**.

## H5 detail (shipped)

### Agent playbook (hosted Path B)

```text
1. ship_pulse → confirm project; note deploy signals (not selfhost_ok loopback)
2. ship_portal provider=cloudflare|vercel|netlify → Sign in (web) / Login CLI URLs
3. Human: Login CLI once (Desktop terminal) OR Sign in (web) + Path A dashboard deploy
4. Prefer Desktop Deployment **Deploy** when auth TTY may appear
5. Else ship_hostdeploy { project, provider } — streams local wrangler/vercel/netlify
6. ship_status / ship_pulse → hosted urls · last_run message [class]
7. On auth → tell human Login CLI / Sign in (web); never paste tokens into chat
8. Open live / Open dashboard (Desktop) → Publish Live check → human Confirm
9. Never claim “Studio created your API token” or fill Create Custom Token
```

### MCP tool

| Tool | Role |
|------|------|
| `ship_hostdeploy` | Thin wrapper on `hostdeploy::run`; provider default `cloudflare`; refuses offline; on Err returns last_run + recovery hint |
| `ship_deploy` | Still Orbit only — do not use for Cloudflare/Vercel/Netlify hosted cut |

Invariant: no secret custody; Prefer Desktop for interactive login prompts (mcp-assist G5).

### Proof (H5)

| Layer | Check |
|-------|--------|
| L1 | `tools_include_ship_hostdeploy` |
| Doc | Playbook above + [mcp-assist-contract](./mcp-assist-contract-design.md) hosted section |
