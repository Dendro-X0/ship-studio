# Evidence — Harbor soft dogfood (Launch L3–L4 · S1.2c surface)

**Date:** 2026-09-28  
**Subject:** `fixtures/harbor` (+ temp wrangler/vercel dir for multi-host primary)  
**shipctl:** `target/release/shipctl.exe` rebuilt after `f2c08f1`  
**Commit under test:** `f2c08f1`

## Setup

1. `cargo build -p shipctl --release`
2. `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1`

## L2 results (CLI)

| Check | Result |
|-------|--------|
| Harbor Launch Payments default | `integrations.panel` **absent** |
| `launch_payments=true` | Payments lane appears · detail mentions opt-in |
| Harbor Launch lanes | `prep` · `targets` · `sign` · `deployment` · `legal` · `cut` (no `payments` until opt-in) |
| `primary_host=cloudflare` on multi-detect fixture | `oauth.hosts` detail includes **Cloudflare (primary)** |
| Scopes | `desktop.desktop` · `docs.website` |
| Cleanup | `harbor-reset`; temp `_dogfood-l4-tmp` removed |

## Desktop UI (operator — remaining)

| Check | How |
|-------|-----|
| Publish copy | Bind Harbor → Publish hints say checklist / Open this step / We'll check… |
| Workflow previews | Dashboard cards show About N steps · ~min |
| Payments toggle | Launch **Payments** off → on rebuilds lane |
| Primary host | Deployment pick Cloudflare → reload → still selected; Launch Verify names that host |

## Verdict

**CLI L2 pass** for Launch L3–L4 on rebuilt release shipctl. **Desktop UI L2** still operator-owned.
