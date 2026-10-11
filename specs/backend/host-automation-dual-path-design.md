# Host automation — dual path (Desktop wizard · MCP)

**Status:** D0–D3 shipped  
**Updated:** 2026-10-07  
**Parents:** [host-deploy-env-import-design](./host-deploy-env-import-design.md) · [mcp-assist-contract-design](./mcp-assist-contract-design.md) · [hosted-deploy-ops-value-bar-design](./hosted-deploy-ops-value-bar-design.md) · [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Surfaces:** Desktop Deployment · `yard_hostdeploy` / `yard_env_*` · `hostdeploy` · `secrets`

```text
GOAL:     Automate vendor deploy/setup steps via local CLI for Tier A+B,
          through Desktop one-click wizard AND MCP — same kernel, human gates.
NOT:      Studio-held API tokens · Create-Token fill · auto-Confirm · DevOps GTM
```

## Locked decisions

1. **Substrate = local vendor CLI** (human Login CLI first). CLIs call vendor backends; Orbit does not store API tokens or PUT secret values over Studio HTTPS.
2. **Scope:** Cloudflare · Vercel · Netlify · Fly · Railway. Tier C–D stay Open/Docs.
3. **Two surfaces, one kernel:** Desktop Deploy/Import and `yard_*` MCP call `hostdeploy` / `secrets` / `portal` / pause-resume.
4. **Human authority:** OAuth, first-time `fly launch` / `railway init|link`, secret values, Publish Confirm.

## Official workflows (vendor tutorials)

| Host | First-time | Steady deploy | Secrets | Auth |
|------|------------|---------------|---------|------|
| Cloudflare Workers | Account · Node · wrangler · login · deploy | `wrangler deploy` | `wrangler secret put` | OAuth |
| Cloudflare Pages | pages project create · pages deploy | `wrangler pages deploy` | `wrangler pages secret` | OAuth |
| Vercel | login · link · deploy --prod | `vercel deploy --prod --yes` | `vercel env add` | OAuth |
| Netlify | login · link/create · deploy --prod --dir | same | `netlify env:import` | OAuth |
| Fly | auth login · **launch** · deploy | `fly deploy` | `fly secrets set NAME=-` | OAuth |
| Railway | login · init/link · **up** | `railway up` | `railway variable set --stdin` | OAuth |

Docs: [Workers CLI](https://developers.cloudflare.com/workers/get-started/guide/) · [Vercel CLI deploy](https://vercel.com/docs/cli/deploying-from-cli) · [Netlify CLI](https://docs.netlify.com/cli/get-started/) · [Fly Launch](https://fly.io/docs/getting-started/launch/) · [fly deploy](https://fly.io/docs/flyctl/deploy/) · [Railway CLI](https://docs.railway.com/guides/cli).

## Dual-path spine

```text
Detect → Auth-check → (optional Import names) → Deploy stream → Result URL
                ↓ fail
         pause [missing_cli|auth|link|put] → human → yard_resume / Retry
```

| Phase | Desktop wizard | MCP |
|-------|----------------|-----|
| Detect | Affinity target + pack | `yard_scopes` / setup |
| Auth | Login CLI on fail | Pause → human login |
| Env | Name checklist → import-env | `yard_env_import` (path+names) · `yard_env_put` |
| Deploy | `hostdeploy` stream | `yard_hostdeploy` (prefer Desktop if TTY) |
| Done | Open live · Confirm | `yard_resume` · human Confirm |

### Pause taxonomy

`missing_cli` · `auth` · `put` · `link` (Fly/Railway first-time) · `deploy` · `confirm`

## Slice plan

| Slice | Scope | Status |
|-------|-------|--------|
| **D0** | This design + research locked | **done** |
| **D1** | MCP: `yard_env_import` (names/path only); `yard_env_put` + import for Fly·Railway | **done** |
| **D2** | Tier B `hostdeploy` (`fly deploy` · `railway up`) + Desktop Deploy dialog | **done** |
| **D3** | Pages `wrangler pages secret`; explicit link gates (no silent `fly launch` / `railway init`) | **done** |
| **P1** | Provider parity vs CF dogfood — [provider-parity-p1-design](./provider-parity-p1-design.md) | **done** |

## Never-say

- “We store your API token”
- “OAuth finished” without Verify/auth-check evidence
- “I pasted your secret into MCP”
- “One-click creates your Fly/Railway account”

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: detect fly/railway plans; Pages put argv; MCP tool schema rejects `value` on import |
| L2 | Desktop: Deploy Fly/Railway card streams CLI; Import still works |
| L3 | MCP: `yard_env_import` dry-run names only; pause on auth |

## Reject

Studio GraphQL/REST with stored tokens · account signup · Tier C–D in this band · DNS/custom domain as Studio job
