# Current session — Ship Studio

**Updated:** 2026-10-04  
**Branch:** `main`  
**Status:** **Uniform CLI host auth vetting** — select CF/Vercel/Netlify/Fly/Railway → same Login CLI gate as Deploy.

## Next Atomic Step

**Idle** — restart Desktop; pick any CLI host card without login → Login CLI opens.

| Option | When |
|--------|------|
| Live Netlify deploy | After auth; set site name in confirm |
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
| OSS / GitHub Releases tool | Ready (v0.2.3 + Desktop dogfood) |
| Customer Payments/Integrations **guide** | Polar · Stripe · Gumroad · Lemon · Paddle · Creem · Waffo |
| Paid Solo checkout on the website | **Optional** — env-gated; not the close-cut |

## Last closed

| Band | Link |
|------|------|
| **Website Netlify bind (S3)** | `apps/website/netlify.toml` · detect + Publish deploy · hostdeploy detect ok; live needs login/link |
| **Client finishability S1–S2** | [client-finishability-design](../../specs/backend/client-finishability-design.md) · PUBLIC_PADDLE + marketing detect · catalog diet · `platforms.host` |
| **Release capability audit** | Client sequences site+payments; CLI=kernel; MCP=agents; zero-curve Out · L2 `apps/website` selfhost ok, host unbound, Paddle env names present |
| **Game butler L3 dogfood** | Installed butler v15.31.0 → `~/.local/bin`; fixture `E:/Temp/ship-butler-dogfood`; `--no-spawn` recipe ok; spawn `spawned:true`; assist itch cue present; **no** `butler_creds` yet (push waits on human login) |
| **Game butler push (CLI + MCP)** | [game-butler-push-design](../../specs/backend/game-butler-push-design.md) · `shipctl butler push` · `ship_butler_push` |
| **Freeze tag** | `freeze/mcp-agent-2026-10-04` — close-cut + MCP G1–G4 + skill |
| **MCP agent skill** | [`.cursor/skills/ship-mcp-agent`](../../.cursor/skills/ship-mcp-agent/SKILL.md) · prefer ship_* over Desktop wizards |
| **S2.4 G3–G4 MCP put/vault honesty** | [mcp-assist-contract](../../specs/backend/mcp-assist-contract-design.md) · TTY bail · vault schema |
| **S2.4 G2 MCP env put launch** | [mcp-assist-contract](../../specs/backend/mcp-assist-contract-design.md) · `ship_env_put` recipe + spawn |
| **S2.4 G1 MCP publish mutations** | [mcp-assist-contract](../../specs/backend/mcp-assist-contract-design.md) · `ship_publish_open`/`verify`/`confirm`/`next` |
| **Close-cut slice 1** | [close-cut-design](../../specs/backend/close-cut-design.md) · legal OSS · Releases-first README |
| **Env Put convenience** | [env-put-convenience-design](../../specs/backend/env-put-convenience-design.md) · Secrets/Human/Integrations Put → host CLI |
| **Integrations Deploy-feel** | [integrations-deploy-parity-design](../../specs/backend/integrations-deploy-parity-design.md) · all lanes `openLinks` |
| **Website Paddle overlay** | [website-paddle-checkout-design](../../specs/backend/website-paddle-checkout-design.md) · [frontend spec](../frontend/website-paddle-checkout-spec.md) · [PADDLE-SETUP.md](../../apps/website/docs/PADDLE-SETUP.md) |
| **Paddle portal minute** | [paddle-portal-minute-design](../../specs/backend/paddle-portal-minute-design.md) · Desktop Open → sandbox dashboard |
| **Creem / Waffo payments guide** | [commerce-creem-waffo-design](../../specs/backend/commerce-creem-waffo-design.md) · [frontend spec](../frontend/payments-creem-waffo-spec.md) · L3 CLI: `shipctl portal --provider creem\|waffo` |
| **S1.6 Cafe / FAQ** | [cafe-faq-design](../../specs/frontend/cafe-faq-design.md) · `/faq` |
| **S1.5 OPERATOR-NEXT product** | [operator-next-product-design](../../specs/frontend/operator-next-product-design.md) · `/honesty` |
| **S1.4 One intent page** | [one-intent-page-design](../../specs/frontend/one-intent-page-design.md) · `/intent` |
| **S1.2 Dashboard honesty** | [dashboard-honesty-design](../../specs/frontend/dashboard-honesty-design.md) |
| **S1.3 Never-say** | [never-say-block-design](../../specs/frontend/never-say-block-design.md) |
| **Desktop UI dogfood** | [evidence-harbor-desktop-dogfood](./evidence-harbor-desktop-dogfood.md) |

## Boot allowlist

1. This file  
2. [CURRENT.md](../CURRENT.md)  
3. [PLATFORMS-AND-PORTAL.md](../product/PLATFORMS-AND-PORTAL.md)  
4. [improvement-backlog.md](../product/improvement-backlog.md)  
