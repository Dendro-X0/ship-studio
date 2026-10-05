# Current session — Orbit Yard

**Updated:** 2026-10-04  
**Branch:** `main`  
**Status:** **W4 payment + refund setup** reopened — sandbox Solo overlay first. Portal/Integrations UX still PAUSED.

## Next Atomic Step

**Paddle Solo sandbox setup** — follow [PADDLE-SETUP.md](../../apps/website/docs/PADDLE-SETUP.md) · dogfood [LICENSE-REFUND-DOGFOOD.md](../../apps/website/docs/LICENSE-REFUND-DOGFOOD.md).

1. Create sandbox API key (`pdl_sdbx_…`) at https://sandbox-vendors.paddle.com/authentication (`product.write` · `price.write` · `client_token.write`)
2. `setx PADDLE_SANDBOX_API_KEY "pdl_sdbx_…"` → new terminal → `python scripts/seed-paddle-solo-sandbox.py`
3. Dashboard: **default payment link** → `http://localhost:4321` (and live site origin when ready)
4. `pnpm website:dev` → `/pricing` test card `4242…` → then license issue / refund revoke dogfood

| Option | When |
|--------|------|
| Put `PUBLIC_PADDLE_*` on Netlify | After local sandbox buy works |
| Confirm Live check on Publish | After Open live looks right |
| Rename Netlify site / GitHub repo | Cosmetic — after Solo path works |

## Design queue (not coding until activated)

| Item | Spec |
|------|------|
| **v0.2.4 release cut** | [v0.2.4-release-plan](../../specs/backend/v0.2.4-release-plan.md) (**shipped**) |
| S3.1 Agent setup / human key confirm | [agent-setup-human-confirm-design](../../specs/backend/agent-setup-human-confirm-design.md) (**`yard_setup` shipped**) |
| S1.6 Cafe / FAQ | [cafe-faq-design](../../specs/frontend/cafe-faq-design.md) (**shipped**) |
| S1.5 Human gates product | [operator-next-product-design](../../specs/frontend/operator-next-product-design.md) (**shipped**) |
| S1.4 One intent page | [one-intent-page-design](../../specs/frontend/one-intent-page-design.md) (**shipped**) |
| S1.2 Dashboard honesty | [dashboard-honesty-design](../../specs/frontend/dashboard-honesty-design.md) (**slice 1 shipped**) |
| S1.3 Never-say | [never-say-block-design](../../specs/frontend/never-say-block-design.md) (**shipped**) |

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| **Vendor handoff coach** | **CANCELLED** |
| **Paddle Solo API keys / overlay dogfood** | **ACTIVE** — maintainer reopened for W4 payment + refund setup |
| **Portal / Integrations UX expansion** | **PAUSED** — obstacle-course verdict |
| Polar paid checkout E2E | Deferred — Polar is not the Solo processor |
| aperio Advanced L4 | Parked |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |
| Drive-by Desktop reliability | Work only via design slices |
| Reliability Later | Non-Windows · **CDP** · vault |

## Release readiness (honest)

| Surface | State |
|---------|--------|
| OSS / GitHub Releases tool | **v0.2.4** Orbit Yard brand — [release](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.4) |
| Customer Payments/Integrations **guide** | Polar · Stripe · Gumroad · Lemon · Paddle · Creem · Waffo |
| Paid Solo checkout on the website | **Optional** — env-gated; not the close-cut |

## Last closed

| Band | Link |
|------|------|
| **v0.2.4 Orbit Yard cut** | Release + installer/zip · CI smoke soft-doctor · [plan](../../specs/backend/v0.2.4-release-plan.md) |
| **Agent setup / human key confirm** | `yard_setup` brief · skill minutes loop · [agent-setup-human-confirm-design](../../specs/backend/agent-setup-human-confirm-design.md) |
| **Rebrand → Orbit Yard** | Product · `orbityard` CLI · `yard_*` MCP · Desktop/website · skill `orbit-yard-mcp-agent`; `.ship/` config dir kept |
| **Uniform CLI host auth vetting** | select CF/Vercel/Netlify/Fly/Railway → Login CLI |
| **Website Netlify bind (S3)** | `apps/website/netlify.toml` |
| **Client finishability S1–S2** | [client-finishability-design](../../specs/backend/client-finishability-design.md) |
| **Game butler push** | `orbityard butler push` · `yard_butler_push` |
| **Freeze tag** | `freeze/mcp-agent-2026-10-04` |
| **MCP agent skill** | [`.cursor/skills/orbit-yard-mcp-agent`](../../.cursor/skills/orbit-yard-mcp-agent/SKILL.md) |

## Boot allowlist

1. This file  
2. [CURRENT.md](../CURRENT.md)  
3. [PLATFORMS-AND-PORTAL.md](../product/PLATFORMS-AND-PORTAL.md)  
