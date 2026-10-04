# Current session — Orbit Yard

**Updated:** 2026-10-04  
**Branch:** `main`  
**Status:** **Rebrand to Orbit Yard** — product, CLI `orbityard`, MCP `orbit_*`, Desktop/website strings. Deploy website next.

## Next Atomic Step

**Deploy Orbit Yard website** (Netlify) after local `pnpm website:build` looks right — then claim `orbityard.dev` if desired.

| Option | When |
|--------|------|
| Rename GitHub repo `ship-studio` → `orbit-yard` | After website live; update clone URLs |
| Live Netlify deploy | Auth + site name; tip: product is Orbit Yard |
| Put PUBLIC_PADDLE_* on Netlify env | Before Solo Buy on the live site |
| Paddle Solo dogfood | Stays PAUSED until explicit reopen |

## Design queue (not coding until activated)

| Item | Spec |
|------|------|
| S1.6 Cafe / FAQ | [cafe-faq-design](../../specs/frontend/cafe-faq-design.md) (**shipped**) |
| S1.5 Human gates product | [operator-next-product-design](../../specs/frontend/operator-next-product-design.md) (**shipped**) |
| S1.4 One intent page | [one-intent-page-design](../../specs/frontend/one-intent-page-design.md) (**shipped**) |
| S1.2 Dashboard honesty | [dashboard-honesty-design](../../specs/frontend/dashboard-honesty-design.md) (**slice 1 shipped**) |
| S1.3 Never-say | [never-say-block-design](../../specs/frontend/never-say-block-design.md) (**shipped**) |

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| **Vendor handoff coach** | **CANCELLED** |
| **Paddle Solo API keys / overlay dogfood** | **PAUSED** — maintainer halt |
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
| OSS / GitHub Releases tool | Ready (v0.2.3 + Desktop dogfood); rebrand pending new release tag |
| Customer Payments/Integrations **guide** | Polar · Stripe · Gumroad · Lemon · Paddle · Creem · Waffo |
| Paid Solo checkout on the website | **Optional** — env-gated; not the close-cut |

## Last closed

| Band | Link |
|------|------|
| **Rebrand → Orbit Yard** | Product · `orbityard` CLI · `orbit_*` MCP · Desktop/website · skill `orbit-yard-mcp-agent`; `.ship/` config dir kept |
| **Uniform CLI host auth vetting** | select CF/Vercel/Netlify/Fly/Railway → Login CLI |
| **Website Netlify bind (S3)** | `apps/website/netlify.toml` |
| **Client finishability S1–S2** | [client-finishability-design](../../specs/backend/client-finishability-design.md) |
| **Game butler push** | `orbityard butler push` · `orbit_butler_push` |
| **Freeze tag** | `freeze/mcp-agent-2026-10-04` |
| **MCP agent skill** | [`.cursor/skills/orbit-yard-mcp-agent`](../../.cursor/skills/orbit-yard-mcp-agent/SKILL.md) |

## Boot allowlist

1. This file  
2. [CURRENT.md](../CURRENT.md)  
3. [PLATFORMS-AND-PORTAL.md](../product/PLATFORMS-AND-PORTAL.md)  
